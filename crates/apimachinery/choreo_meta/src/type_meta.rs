use serde::{Deserialize, Serialize};

/// Type information that is flattened into every object
#[derive(Deserialize, Serialize, Clone, Default, Debug, Eq, PartialEq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct TypeMeta {
    /// The version of the API
    pub api_version: String,
    /// The name of the API
    pub kind: String,
}
