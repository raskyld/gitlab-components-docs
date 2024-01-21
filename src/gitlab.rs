//! This module is using the [serde framework](https://serde.rs)
//! with the serde_yaml crate to be able to interpret the GitLab config
//! files.
//!
//! Reference: https://docs.gitlab.com/ee/ci/yaml/

use std::collections::BTreeMap;

use crate::gitlab;
use crate::gitlab::LoadingResult::{Failed, Success};
use log::warn;
use serde::{Deserialize, Serialize};
use std::fs::{read_dir, read_to_string};
use std::path::{Path, PathBuf};

#[derive(Deserialize, Serialize, Debug)]
pub struct Components {
    pub spec: Spec,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Spec {
    pub inputs: BTreeMap<String, Input>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Input {
    pub default: Option<String>,
    pub description: Option<String>,
    pub options: Option<Vec<String>>,

    #[serde(rename = "type")]
    pub type_t: Option<String>,
}

/// Returned by load_components() for each file visited
pub enum LoadingResult {
    Success(Components),
    Failed(Vec<String>),
}

pub fn load_components() -> std::io::Result<BTreeMap<String, LoadingResult>> {
    read_dir("templates/").map(|dir_entries| {
        let mut results: BTreeMap<String, LoadingResult> = BTreeMap::new();
        for dir_entry in dir_entries {
            let dir_entry = match dir_entry {
                Ok(dir_entry) => dir_entry,
                Err(err) => {
                    warn!("could not read an entry: {}", err);
                    continue;
                }
            };

            let path = dir_entry.path();
            let Some(file_stem) = path.file_stem() else {
                warn!(
                    "skipping {}: could not determinate the component name",
                    path.display()
                );
                continue;
            };

            let Some(file_stem_str) = file_stem.to_str() else {
                warn!(
                    "skipping {}: path contains non UTF-8 characters",
                    path.display()
                );
                continue;
            };

            load_component(&path).map_or_else(
                |err| warn!("could not process {}: {}", path.display(), err.to_string()),
                |loading_result| {
                    results.insert(file_stem_str.to_string(), loading_result);
                },
            );
        }
        results
    })
}

pub fn load_component(path: &Path) -> std::io::Result<LoadingResult> {
    let mut path_buf: PathBuf;
    let path = if path.is_dir() {
        path_buf = path.to_path_buf();
        path_buf.push("template.yml");
        &path_buf
    } else {
        path
    };

    read_to_string(path).map(|content| {
        let mut warnings: Vec<String> = vec![];
        for doc in serde_yaml::Deserializer::from_str(content.as_str()) {
            match gitlab::Components::deserialize(doc) {
                Ok(found_comp) => {
                    return Success(found_comp);
                }
                Err(err) => warnings.push(err.to_string()),
            }
        }
        Failed(warnings)
    })
}
