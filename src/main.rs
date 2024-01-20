mod gitlab;

use std::collections::{HashMap};
use std::env::current_dir;


use log::{error, info, warn};
use tera::{Context, Tera, Value};
use clap::Parser;

use crate::gitlab::Components;

const DEFAULT_README_TPL: &'static str = r####"
# {{ catalog_name }}

{{ catalog_desc }}

## Components

{% for comp in components -%}
{% for in_name, input in comp.spec.inputs -%}
### {{ in_name }}

{{ inputs_table(inputs=comp.spec.inputs) }}
{%- endfor %}
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

    ctx.insert("catalog_desc", &cli.catalog_desc.unwrap_or("A super GitLab CI/CD catalog!".to_owned()));

    let mut components: Vec<Components> = vec![];
    match gitlab::load_components() {
        Ok(results) => for (name, loading_result) in results {
            match loading_result {
                gitlab::LoadingResult::Failed(warnings) => warn!("could not load {}: {}", name, warnings.join(", ")),
                gitlab::LoadingResult::Success(comp) => components.push(comp)
            }
        },
        Err(err) => error!("could not load components: {}", err.to_string())
    };

    ctx.insert("components", &components);

    info!("content is: {}", engine.render("README.md.tera", &ctx).unwrap())
}

fn create_engine() -> Tera {
    let mut engine = Tera::default();
    engine.add_template_file("README.md.tera", None).unwrap_or_else(|err| {
            warn!("failed to load README.md.tera: {}", err.to_string());
            info!("Using sensible default for the README!");
            // NB(raskyld): it's fine to panic in this case as
            // we are not supposed to release a compiled binary with a malformed
            // built-in template
            engine.add_raw_template("README.md.tera", DEFAULT_README_TPL).unwrap();
    });

    engine.register_function("inputs_table", inputs_table);
    engine
}

fn inputs_table(_: &HashMap<String, Value>) -> tera::Result<Value> {
    Ok(Value::from("test"))
}
