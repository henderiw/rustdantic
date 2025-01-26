use ::default_derive::Default as ChoreoDefault;
use ::validate_derive::Validate as ChoreoValidate;
use ::serde::{Serialize, Deserialize};

use crate::ObjectReference;

#[derive(ChoreoDefault, ChoreoValidate, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RelationReference {
    pub object_reference: ObjectReference,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<std::collections::BTreeMap<String, String>>,

}