use chrono::{DateTime, Utc};
use ::default_derive::Default as ChoreoDefault;
use ::validate_derive::Validate as ChoreoValidate;
use ::serde::{Serialize, Deserialize};

/// Condition contains details for one aspect of the current state of this API Resource.
#[derive(ChoreoDefault, ChoreoValidate, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Condition {
    #[cdefault("none")]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// lastTransitionTime is the last time the condition transitioned from one status to another. This should be when the underlying condition changed.  If that is not known, then using the time when the API field changed is acceptable.
    pub last_transition_time: Option<DateTime<Utc>>,

    /// message is a human readable message indicating details about the transition. This may be an empty string.
    pub message: String,

    /// observedGeneration represents the .metadata.generation that the condition was set based upon. For instance, if .metadata.generation is currently 12, but the .status.conditions\[x\].observedGeneration is 9, the condition is out of date with respect to the current state of the instance.
    //#[serde(skip_serializing_if = "Option::is_none")]
    pub observed_generation: Option<i64>,

    /// reason contains a programmatic identifier indicating the reason for the condition's last transition. Producers of specific condition types may define expected values and meanings for this field, and whether the values are considered a guaranteed API. The value should be a CamelCase string. This field may not be empty.
    pub reason: String,

    /// status of the condition, one of True, False, Unknown.
    pub status: String,

    /// type of condition in CamelCase or in foo.example.com/CamelCase.
    pub type_: String,
}

#[derive(ChoreoDefault, ChoreoValidate, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ConditionStatus {
    pub conditions: Vec<Condition>,
} 