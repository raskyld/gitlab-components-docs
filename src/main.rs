mod gitlab;

use std::collections::HashMap;
use std::env::current_dir;
use std::fs::read_dir;
use log::{error, info, warn};
use tera::{Context, Tera, Value};
use clap::Parser;

const DEFAULT_README_TPL: &'static str = r####"
# {{ catalog_name }}

{{ catalog_desc }}

## Components

{% for comp in components -%}
### {{ comp.name }}

{{ inputs_table(inputs=comp.inputs) }}
{%- endfor %}
"####;

/// A simple CLI to generate docs for your GitLab CI/CD components
#[derive(Parser)]
struct Cli {
    /// Which title to use (default to the current working directory)
    #[arg(short='n',long)]
    catalog_name: Option<String>,

    /// A description to add on your README header
    #[arg(short='d',long)]
    catalog_desc: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    env_logger::init();

    let engine = create_engine();
    let mut ctx = Context::new();

    // Try filling the catalog name using the current working directory if needed
    // NB(raskyld): working directories with non-UTF8 chars are not supported yet.
    match cli.catalog_name {
        Some(str) => ctx.insert("catalog_name", &str),
        None => {
            let pwd = current_dir();

            if pwd.is_err() {
                error!("unable to read the current working directory: {}", pwd.err().unwrap());
                ctx.insert("catalog_name", "UNKNOWN_TITLE");
            } else {
                ctx.insert("catalog_name", pwd.unwrap().file_name().unwrap().to_str().unwrap_or_else(|| {
                    error!("your current working directory contains non UTF8 characters");
                    "UNKNOWN_TITLE"
                }))
            }
        }
    };

    ctx.insert("catalog_desc", cli.catalog_desc.map_or("A super GitLab CI/CD catalog!", |a| &Box::new(a)));

    if let dir = read_dir("./templates").unwrap() {
        dir.collect()
    }
}

fn create_engine() -> Tera {
    let mut engine = match Tera::new("*.md.tera") {
        Ok(tera) => tera,
        Err(e) => {
            warn!("glob *.md.tera failed: {}", e.to_string());
            info!("Using sensible default for the README!");
            // NB(raskyld): it's fine to panic in this case as
            // we are not supposed to release a compiled binary with a malformed
            // built-in template
            let mut tera = Tera::default();
            tera.add_raw_template("README.md.tera", DEFAULT_README_TPL).unwrap();
            return tera
        }
    };

    engine.register_function("inputs_table", inputs_table);

    engine
}

fn inputs_table(_: &HashMap<String, Value>) -> tera::Result<Value> {
    Ok(Value::from("test"))
}
