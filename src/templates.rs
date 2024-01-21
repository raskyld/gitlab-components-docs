use log::{info, warn};
use std::collections::HashMap;
use tera::{Error, Tera, Value};

pub const DEFAULT_README_TPL: &str = r####"
{% import "macros" as macros %}
# {{ catalog_name }}

{{ catalog_desc }}

## Components

[[_TOC_]]

{% for name, comp in components -%}
### {{ name }}

{{ macros::inputs_table(inputs=comp.spec.inputs) }}
{%- endfor %}
"####;

pub const ENTRYPOINT_TEMPLATE: &str = r####"
{% include "README.md.tera" %}
{% if footer_enabled -%}
---

Generated with [raskyld/gitlab-components-docs](https://github.com/raskyld/gitlab-components-docs) :purple_heart:

Version: `{{ version }}`.
{%- endif %}
"####;

pub const MACROS: &str = r####"
{% macro inputs_table(inputs) -%}
| Name | Type | Description | Default |
| --- | --- | --- | --- |
{% for name, input in inputs -%}
| `{{ name }}` | `{{ input.type | default(value="string") }}` | {{ input.description | trim_newline }} | `{{ input.default }}` |
{% endfor %}
{%- endmacro inputs_table %}
"####;

pub fn create_engine() -> Tera {
    let mut engine = Tera::default();

    engine.register_filter("trim_newline", filter_trim_newline);

    engine
        .add_raw_template("entrypoint", ENTRYPOINT_TEMPLATE)
        .expect("this binary is non-functional");

    engine
        .add_raw_template("macros", MACROS)
        .expect("this binary is non-functional");

    engine
        .add_template_file("README.md.tera", None)
        .unwrap_or_else(|err| {
            warn!("failed to load README.md.tera: {}", err.to_string());
            info!("Using sensible default for the README!");
            // NB(raskyld): it's fine to panic in this case as
            // we are not supposed to release a compiled binary with a malformed
            // built-in template
            engine
                .add_raw_template("README.md.tera", DEFAULT_README_TPL)
                .unwrap();
        });

    engine
}

fn filter_trim_newline(val: &Value, _: &HashMap<String, Value>) -> tera::Result<Value> {
    val.as_str()
        .ok_or(Error::from("could not read passed value"))
        .map(|str| Value::from(str.replace('\n', " ")))
}
