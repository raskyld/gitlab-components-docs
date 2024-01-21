mod gitlab;
mod templates;

use std::collections::BTreeMap;
use std::env::current_dir;
use std::fs::File;
use std::io;
use std::io::{stdout, Write};
use std::process::exit;

use crate::gitlab::Components;
use clap::Parser;
use log::{error, warn};
use tera::Context;

const DEFAULT_CATALOG_NAME: &str = "Unnamed GitLab CI/CD Catalog";
const DEFAULT_CATALOG_DESC: &str = "A super GitLab CI/CD Catalog";

/// A simple CLI to generate docs for your GitLab CI/CD components
#[derive(Parser)]
struct Cli {
    /// Which title to use (default to the current working directory)
    #[arg(short = 'n', long)]
    catalog_name: Option<String>,

    /// A description to add on your README header
    #[arg(short = 'd', long)]
    catalog_desc: Option<String>,

    /// Remove the footer added at the end of the README.md
    #[arg(long)]
    no_footer: bool,

    /// Write to stdout instead of writing to README.md
    #[arg(long)]
    dry_run: bool,
}

fn main() {
    let cli = Cli::parse();

    env_logger::init();

    let engine = templates::create_engine();
    let mut ctx = Context::new();

    // Try filling the catalog name using the current working directory if needed
    // NB(raskyld): working directories with non-UTF8 chars are not supported yet.
    match cli.catalog_name {
        Some(str) => ctx.insert("catalog_name", &str),
        None => {
            let pwd = current_dir();

            ctx.insert(
                "catalog_name",
                match &pwd {
                    Ok(path) => path.file_name().unwrap().to_str().unwrap_or_else(|| {
                        error!("your current working directory contains non UTF8 characters");
                        DEFAULT_CATALOG_NAME
                    }),
                    Err(err) => {
                        error!("unable to read the current working directory: {}", err);
                        DEFAULT_CATALOG_NAME
                    }
                },
            );
        }
    };

    ctx.insert(
        "catalog_desc",
        cli.catalog_desc.as_deref().unwrap_or(DEFAULT_CATALOG_DESC),
    );

    let mut components: BTreeMap<String, Components> = BTreeMap::new();
    match gitlab::load_components() {
        Ok(results) => {
            for (name, loading_result) in results {
                match loading_result {
                    gitlab::LoadingResult::Failed(warnings) => {
                        warn!("could not load {}: {}", name, warnings.join(", "));
                    }
                    gitlab::LoadingResult::Success(comp) => {
                        components.insert(name, comp);
                    }
                };
            }
        }
        Err(err) => error!("could not load components: {}", err.to_string()),
    };

    ctx.insert("components", &components);
    ctx.insert("version", get_version());
    ctx.insert("footer_enabled", &!cli.no_footer);

    let rendered = match engine.render("entrypoint", &ctx) {
        Ok(rendered) => rendered,
        Err(err) => {
            error!("failed to render README.md: {}", err);
            exit(1);
        }
    };

    let mut writer: Box<dyn Write> = if cli.dry_run {
        Box::new(stdout())
    } else {
        match File::create("README.md") {
            Ok(file) => Box::new(file),
            Err(err) => {
                error!("failed to open README.md: {}", err.to_string());
                exit(1);
            }
        }
    };

    'retry_write: loop {
        match Write::write(&mut writer, rendered.as_bytes()) {
            Err(err) => match err.kind() {
                io::ErrorKind::Interrupted => continue 'retry_write,
                _ => {
                    error!("failed to write the result: {}", err.to_string());
                    exit(2);
                }
            },
            Ok(_) => {
                Write::flush(&mut writer).unwrap_or_else(|err| {
                    error!("failed to flush the result: {}", err.to_string());
                    exit(3);
                });
                break 'retry_write;
            }
        }
    }
}

#[cfg(debug_assertions)]
fn get_version() -> &'static str {
    "dev"
}

#[cfg(not(debug_assertions))]
fn get_version() -> &'static str {
    option_env!("CARGO_PKG_VERSION").unwrap_or("unknown")
}
