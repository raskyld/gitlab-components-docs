//! This module is using the [serde framework](https://serde.rs)
//! with the serde_yaml crate to be able to interpret the GitLab config
//! files.
//!
//! Reference: https://docs.gitlab.com/ee/ci/yaml/

use std::collections::BTreeMap;
use std::fs::{read_dir, read_to_string};
use std::path::PathBuf;
use log::{debug, warn};
use serde::{Deserialize, Serialize};
use crate::gitlab;
use crate::gitlab::LoadingResult::{Failed, Success};

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
    Failed(Vec<String>)
}

pub fn load_components() -> std::io::Result<BTreeMap<String, LoadingResult>> {
    read_dir("templates/").map(|dir_entries| {
        let mut results: BTreeMap<String, LoadingResult> = BTreeMap::new();
        for dir_entry in dir_entries {
            match dir_entry {
                Ok(dir_entry) => {
                    let path = dir_entry.path();
                    load_component(path.clone()).map_or_else(
                        |err| warn!("could not process {}: {}", path.display(), err.to_string()),
                        |loading_result| {
                            results.insert(path.display().to_string(), loading_result);
                        }
                    );
                },
                Err(err) => warn!("could not read an entry: {}", err),
            };
        };
        return results;
    })
}

pub fn load_component(mut path: PathBuf) -> std::io::Result<LoadingResult> {
    let path_str = path.display().to_string();

    if path.is_dir() {
        path.push("template.yml")
    }

    debug!("processing {}", path_str);
    read_to_string(path).map(|content| {
        let mut warnings: Vec<String> = vec![];
        for doc in serde_yaml::Deserializer::from_str(content.as_str()) {
            match gitlab::Components::deserialize(doc) {
                Ok(found_comp) => {
                    return Success(found_comp);
                },
                Err(err) => warnings.push(err.to_string())
            }
        }
        return Failed(warnings)
    })
}
