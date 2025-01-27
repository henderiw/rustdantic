use ::choreo_meta::{TypeMeta, ObjectMeta, GroupVersionKind};

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct Unstructured {
    /// The type fields, not always present
    #[serde(flatten, default)]
    pub types: Option<TypeMeta>,
    /// Object metadata
    #[serde(default)]
    pub metadata: ObjectMeta,

    /// All other keys
    #[serde(flatten)]
    pub data: serde_json::Value,
}

impl Unstructured {
    // constructors
    pub fn new(gvk: &GroupVersionKind, name: &str) -> Self {
        Self {
            types: Some(TypeMeta {
                api_version: gvk.api_version(),
                kind: gvk.kind.clone(),
            }),
            metadata: ObjectMeta {
                name: Some(name.to_string()),
                ..Default::default()
            },
            data: serde_json::Value::Null,
        }
    }

    pub fn from_json(input: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(input)
    }

    pub fn from_yaml(input: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(input)
    }
}

// serialize
impl Unstructured {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self)
    }

    pub fn to_yaml(&self) -> Result<String, serde_yaml::Error> {
        serde_yaml::to_string(&self)
    }
}

// getters
impl Unstructured {
    /// Get the `apiVersion` of the object
    pub fn api_version(&self) -> Option<&str> {
        self.types.as_ref().map(|t| t.api_version.as_str())
    }

    /// Get the `kind` of the object
    pub fn kind(&self) -> Option<&str> {
        self.types.as_ref().map(|t| t.kind.as_str())
    }

    /// Get the `group` part of the `apiVersion`
    pub fn group(&self) -> Option<&str> {
        self.api_version()
            .and_then(|api_version| api_version.split('/').next())
    }

    /// Get the `version` part of the `apiVersion`
    pub fn version(&self) -> Option<&str> {
        self.api_version()
            .and_then(|api_version| api_version.split('/').nth(1))
    }

    pub fn gvk(&self) -> GroupVersionKind {
        GroupVersionKind{
            group: self.group().unwrap_or("").to_string(),
            version: self.version().unwrap_or("").to_string(),
            kind: self.kind().unwrap_or("").to_string(),
        }
    }

    /// Get the metadata of the object
    pub fn metadata(&self) -> &ObjectMeta {
        &self.metadata
    }

    /// Get the data of the object
    pub fn data(&self) -> &serde_json::Value {
        &self.data
    }

    /// Get a specific field from `data` by key
    pub fn data_field(&self, key: &str) -> Option<&serde_json::Value> {
        self.data.get(key)
    }
}

// setters

impl Unstructured {
    /// Set the `apiVersion` of the object
    pub fn set_api_version(&mut self, api_version: impl Into<String>) {
        if let Some(types) = &mut self.types {
            types.api_version = api_version.into();
        } else {
            self.types = Some(TypeMeta {
                api_version: api_version.into(),
                kind: String::new(),
            });
        }
    }

    /// Set the `kind` of the object
    pub fn set_kind(&mut self, kind: impl Into<String>) {
        if let Some(types) = &mut self.types {
            types.kind = kind.into();
        } else {
            self.types = Some(TypeMeta {
                api_version: String::new(),
                kind: kind.into(),
            });
        }
    }

    /// Set the metadata of the object
    pub fn set_metadata(&mut self, metadata: ObjectMeta) {
        self.metadata = metadata;
    }

    /// Set the data of the object
    pub fn set_data(&mut self, data: serde_json::Value) {
        self.data = data;
    }

    /// Set a specific field in `data`
    pub fn set_data_field(&mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) {
        if let serde_json::Value::Object(map) = &mut self.data {
            map.insert(key.into(), value.into());
        } else {
            let mut map = serde_json::Map::new();
            map.insert(key.into(), value.into());
            self.data = serde_json::Value::Object(map);
        }
    }
}