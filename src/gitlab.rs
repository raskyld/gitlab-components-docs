use std::collections::BTreeMap;

// Using https://docs.gitlab.com/ee/ci/yaml/ as a reference
pub struct Components {
    spec: Option<Spec>,
}

pub struct Spec {
    inputs: Option<BTreeMap<String, Input>>,
}

pub struct Input {
    default: Option<String>,

}
