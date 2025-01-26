use ::default_derive::Default as ChoreoDefault;
use ::validate_derive::Validate as ChoreoValidate;
use ::serde::{Serialize, Deserialize};

#[derive(ChoreoDefault, ChoreoValidate, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ObjectReference {
    /// API version of the referent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_version: Option<String>,

    /// If referring to a piece of an object instead of an entire object, this string should contain a valid JSON/Go field access statement, such as desiredState.manifest.containers\[2\]. For example, if the object reference is to a container within a pod, this would take on a value like: "spec.containers{name}" (where "name" refers to the name of the container that triggered the event) or if no container name is specified "spec.containers\[2\]" (container with index 2 in this pod). This syntax is chosen only to have some well-defined way of referencing a part of an object.
    ///pub field_path: Option<String>,

    /// Kind of the referent. More info: https://git.k8s.io/community/contributors/devel/sig-architecture/api-conventions.md#types-kinds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,

    /// Name of the referent. More info: https://kubernetes.io/docs/concepts/overview/working-with-objects/names/#names
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Namespace of the referent. More info: https://kubernetes.io/docs/concepts/overview/working-with-objects/namespaces/
    ///pub namespace: Option<String>,

    /// Specific resourceVersion to which this reference is made, if any. More info: https://git.k8s.io/community/contributors/devel/sig-architecture/api-conventions.md#concurrency-control-and-consistency
    ///pub resource_version: Option<String>,

    /// UID of the referent. More info: https://kubernetes.io/docs/concepts/overview/working-with-objects/names/#uids
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uid: Option<String>,
}