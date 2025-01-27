use std::borrow::Cow;
use choreo_meta::ObjectMeta;

pub trait Resource {
    /// Type information for types that do not know their resource information at compile time.
    ///
    /// Types that know their metadata at compile time should select `DynamicType = ()`.
    /// Types that require some information at runtime should select `DynamicType`
    /// as type of this information.
    ///
    /// See [`DynamicObject`](crate::dynamic::DynamicObject) for a valid implementation of non-k8s-openapi resources.
    type DynamicType: Send + Sync + 'static;

    /// Returns kind of this object
    fn kind(dt: &Self::DynamicType) -> Cow<'_, str>;
    /// Returns group of this object
    fn group(dt: &Self::DynamicType) -> Cow<'_, str>;
    /// Returns version of this object
    fn version(dt: &Self::DynamicType) -> Cow<'_, str>;
    /// Returns apiVersion of this object
    fn api_version(dt: &Self::DynamicType) -> Cow<'_, str> {
        api_version_from_group_version(Self::group(dt), Self::version(dt))
    }
    /// Returns the plural name of the kind
    fn plural(dt: &Self::DynamicType) -> Cow<'_, str>;
    // Metadata that all persisted resources must have
    fn meta(&self) -> &ObjectMeta;
    // Metadata that all persisted resources must have
    fn meta_mut(&mut self) -> &mut ObjectMeta;

    /// The type of the `spec` of this resource
    type Spec;

    /// Returns a reference to the `spec` of the object
    fn spec(&self) -> &Self::Spec;

    /// Returns a mutable reference to the `spec` of the object
    fn spec_mut(&mut self) -> &mut Self::Spec;

    /// The type of the `status` object
    type Status;

    /// Returns an optional reference to the `status` of the object
    fn status(&self) -> Option<&Self::Status>;

    /// Returns an optional mutable reference to the `status` of the object
    fn status_mut(&mut self) -> &mut Option<Self::Status>;
}

/// Helper function that creates the `apiVersion` field from the group and version strings.
pub fn api_version_from_group_version<'a>(
    group: Cow<'a, str>,
    version: Cow<'a, str>,
) -> Cow<'a, str> {
    // this is not supported, actually in choreo, but k8s uses this for its core types
    if group.is_empty() {
        return version;
    }

    let mut output = group;
    output.to_mut().push('/');
    output.to_mut().push_str(&version);
    output
}
