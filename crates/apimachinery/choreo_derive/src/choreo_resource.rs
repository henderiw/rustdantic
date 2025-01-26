use darling::{FromDeriveInput, FromMeta};
use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt as _};
use syn::{parse_quote, Data, DeriveInput, Path};

/// Values we can parse from #[choreo(attrs)]
/// allows the user to specify specific metadata for the resource
#[derive(Debug, FromDeriveInput)]
#[darling(attributes(choreo))]
struct ChoreoResourceAttrs {
    group: String,
    version: String,
    kind: String,
    status_name: String,
    #[darling(rename = "root")]
    /// kind_struct defines the name of the root struct
    kind_struct: Option<String>,
    /// lowercase plural of kind (inferred if omitted)
    plural: Option<String>,
    /// singular defaults to lowercased kind
    singular: Option<String>,
    // derives allows you to specify traits you want this resource to implement/comply to
    #[darling(multiple, rename = "derive")]
    derives: Vec<String>,
    #[darling(default)]
    crates: Crates,
    #[darling(multiple, rename = "annotation")]
    annotations: Vec<KVTuple>,
    #[darling(multiple, rename = "label")]
    labels: Vec<KVTuple>,
}

#[derive(Debug)]
struct KVTuple(String, String);

impl FromMeta for KVTuple {
    fn from_list(items: &[darling::ast::NestedMeta]) -> darling::Result<Self> {
        if items.len() == 2 {
            if let (
                darling::ast::NestedMeta::Lit(syn::Lit::Str(key)),
                darling::ast::NestedMeta::Lit(syn::Lit::Str(value)),
            ) = (&items[0], &items[1])
            {
                return Ok(KVTuple(key.value(), value.value()));
            }
        }

        Err(darling::Error::unsupported_format(
            "expected `\"key\", \"value\"` format",
        ))
    }
}

impl From<(&'static str, &'static str)> for KVTuple {
    fn from((key, value): (&'static str, &'static str)) -> Self {
        Self(key.to_string(), value.to_string())
    }
}

impl ToTokens for KVTuple {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let (k, v) = (&self.0, &self.1);
        tokens.append_all(quote! { (#k, #v) });
    }
}

#[derive(Debug, FromMeta)]
struct Crates {
    #[darling(default = "Self::default_choreo_core")]
    choreo_core: Path,
    #[darling(default = "Self::default_choreo_meta")]
    choreo_meta: Path,
    #[darling(default = "Self::default_serde")]
    serde: Path,
    #[darling(default = "Self::default_serde_json")]
    serde_json: Path,
    #[darling(default = "Self::default_std")]
    std: Path,
}

// Default is required when the subattribute isn't mentioned at all
// Delegate to darling rather than deriving, so that we can piggyback off the `#[darling(default)]` clauses
impl Default for Crates {
    fn default() -> Self {
        Self::from_list(&[]).unwrap()
    }
}

impl Crates {
    fn default_choreo_core() -> Path {
        parse_quote! { ::choreo_core }
    }

    fn default_choreo_meta() -> Path {
        parse_quote! { ::choreo_meta }
    }

    fn default_serde() -> Path {
        parse_quote! { ::serde }
    }

    fn default_serde_json() -> Path {
        parse_quote! { ::serde_json }
    }

    fn default_std() -> Path {
        parse_quote! { ::std }
    }
}


pub(crate) fn derive(input: TokenStream) -> TokenStream {
    let derive_input: DeriveInput = match syn::parse2(input) {
        Err(err) => return err.to_compile_error(),
        Ok(di) => di,
    };

    // Limit derive to structs
    match derive_input.data {
        Data::Struct(_) | Data::Enum(_) => {}
        _ => {
            return syn::Error::new_spanned(
                &derive_input.ident,
                r#"Unions can not #[derive(ChoreoResource)]"#,
            )
            .to_compile_error()
        }
    }

    let choreo_resource_attrs = match ChoreoResourceAttrs::from_derive_input(&derive_input) {
        Err(err) => return err.write_errors(),
        Ok(attrs) => attrs,
    };

    #[allow(unused_variables)]
    let ChoreoResourceAttrs {
        group,
        version,
        kind,
        kind_struct,
        plural,
        singular,
        derives,
        status_name,
        crates:
            Crates {
                choreo_core,
                choreo_meta,
                serde,
                serde_json,
                std,
            },
        annotations,
        labels,
    } = choreo_resource_attrs;

    let struct_name = kind_struct.clone().unwrap_or_else(|| kind.clone());
    if derive_input.ident == struct_name {
        return syn::Error::new_spanned(
            derive_input.ident,
            r#"#[derive(ChoreoResource)] `kind = "..."` must not equal the struct name (this is generated)"#,
        )
        .to_compile_error();
    }
    let visibility = derive_input.vis;
    let spec_ident = derive_input.ident;

    // Create a new root object
    let root_ident = Ident::new(&struct_name, Span::call_site());
    // Create a new status object
    let status_ident = Ident::new(&status_name, Span::call_site());
    

    //eprintln!("rootident_str {:?}", rootident_str);

    
    let mut derive_paths: Vec<Path> = vec![
        syn::parse_quote! { #serde::Deserialize },
        syn::parse_quote! { Clone },
        syn::parse_quote! { Debug },
    ];
    let mut has_default = false;
    for d in &derives {
        if d == "Default" {
            has_default = true; // overridden manually to avoid confusion
        } else {
            match syn::parse_str(d) {
                Err(err) => return err.to_compile_error(),
                Ok(d) => derive_paths.push(d),
            }
        }
    }
    

    // Enable schema generation.
    // let schema_mode = schema_mode.unwrap_or(SchemaMode::Derived);
    // We exclude fields `apiVersion`, `kind`, and `metadata` from our schema because
    // these are validated by the API server implicitly. Also, we can't generate the
    // schema for `metadata` (`ObjectMeta`) because it doesn't implement `JsonSchema`.
    // let schemars_skip = schema_mode.derive().then_some(quote! { #[schemars(skip)] });
    let choreo_meta_annotations = if !annotations.is_empty() {
        quote! { Some(std::collections::BTreeMap::from([#((#annotations.0.to_string(), #annotations.1.to_string()),)*])) }
    } else {
        quote! { None }
    };

    let choreo_meta_labels = if !labels.is_empty() {
        quote! { Some(std::collections::BTreeMap::from([#((#labels.0.to_string(), #labels.1.to_string()),)*])) }
    } else {
        quote! { None }
    };

    // 1. generate the impl for the root resource with spec
    let root_obj = generate_root_object(
        &visibility,
        &root_ident,
        &spec_ident,
        &status_ident,
        &choreo_meta,
        &serde,
        &choreo_meta_annotations,
        &choreo_meta_labels,
    );

    //eprintln!("Generated code root_obj: \n{}", root_obj);

    let name = singular.unwrap_or_else(|| kind.to_ascii_lowercase());
    let plural: String = plural.unwrap_or_else(|| to_plural(&name));
    let impl_resource = generate_resource_trait_impl(
        &root_ident,
        &spec_ident,
        &status_ident,
        &choreo_core,
        &choreo_meta,
        group.as_str(),
        version.as_str(),
        kind.as_str(),
        plural.as_str(),
    );

    let impl_default = generate_default_trait_impl(
        &root_ident,
        has_default,
        &choreo_meta,
    );

    quote! {
        #root_obj
        #impl_resource
        #impl_default
    }
}

fn generate_root_object(
    visibility: &syn::Visibility,
    root_ident: &Ident,
    spec_ident: &Ident,
    status_ident: &Ident,
    choreo_meta: &Path,
    serde: &Path,
    annotations: &TokenStream,
    labels: &TokenStream,
) -> TokenStream {
    //let root_ident_str = root_ident.to_string();
    let quoted_serde = Literal::string(&serde.to_token_stream().to_string());

    quote! {
        #[automatically_derived]
        #[allow(missing_docs)]
        #[derive(
            serde::Serialize,
            serde::Deserialize, 
            Clone, 
            Debug,
            ChoreoDefault,
            ChoreoValidate,
        )]
        #[serde(rename_all = "camelCase")]
        #[serde(crate = #quoted_serde)]
        #visibility struct #root_ident {
            #visibility metadata: #choreo_meta::ObjectMeta,
            #visibility spec: #spec_ident,
            #[serde(skip_serializing_if = "Option::is_none")]
            #visibility status: Option<#status_ident>,
        }

        impl #root_ident {
            pub fn new(name: &str, spec: #spec_ident) -> Self {
                Self {
                    metadata: #choreo_meta::ObjectMeta {
                        annotations: #annotations,
                        labels: #labels,
                        name: Some(name.to_string()),
                        ..Default::default()
                    },
                    spec: spec,
                    status: None, // can also be implemented through the defaulter
                }
            }
        }
        /* 
        impl #serde::Serialize for #root_ident {
            fn serialize<S: #serde::Serializer>(&self, ser: S) -> std::result::Result<S::Ok, S::Error> {
                use #serde::ser::SerializeStruct;
                let mut obj = ser.serialize_struct(#root_ident_str, 4 )?;
                obj.serialize_field("apiVersion", &<#root_ident as choreo_core::Resource>::api_version(&()))?;
                obj.serialize_field("kind", &<#root_ident as choreo_core::Resource>::kind(&()))?;
                obj.serialize_field("metadata", &self.metadata)?;
                obj.serialize_field("spec", &self.spec)?;
                obj.serialize_field("status", &self.status)?;
                obj.end()
            }
        }
        */
    }
}

fn generate_resource_trait_impl(
    root_ident: &Ident,
    spec_ident: &Ident,
    status_ident: &Ident,
    choreo_core: &Path,
    choreo_meta: &Path,
    group: &str,
    version: &str,
    kind: &str,
    plural: &str,
) -> TokenStream {
    let api_ver = format!("{group}/{version}");
    quote! {
        impl #choreo_core::Resource for #root_ident {
            type DynamicType = ();

            fn group(_: &()) -> std::borrow::Cow<'_, str> {
                #group.into()
            }

            fn kind(_: &()) -> std::borrow::Cow<'_, str> {
                #kind.into()
            }

            fn version(_: &()) -> std::borrow::Cow<'_, str> {
                #version.into()
            }

            fn api_version(_: &()) -> std::borrow::Cow<'_, str> {
                #api_ver.into()
            }

            fn plural(_: &()) -> std::borrow::Cow<'_, str> {
                #plural.into()
            }

            fn meta(&self) -> &#choreo_meta::ObjectMeta {
                &self.metadata
            }

            fn meta_mut(&mut self) -> &mut #choreo_meta::ObjectMeta {
                &mut self.metadata
            }

            type Spec = #spec_ident;

            fn spec(&self) -> &#spec_ident {
                &self.spec
            }

            fn spec_mut(&mut self) -> &mut #spec_ident {
                &mut self.spec
            }

            type Status = #status_ident;

            fn status(&self) -> Option<&#status_ident> {
                self.status.as_ref()
            }

            fn status_mut(&mut self) -> &mut Option<#status_ident> {
                &mut self.status
            }

        }
    }
}

fn generate_default_trait_impl(
    rootident: &Ident,
    has_default: bool,
    choreo_meta: &Path,
) -> TokenStream {
    if has_default {
        quote! {
            impl Default for #rootident {
                fn default() -> Self {
                    Self {
                        metadata: #choreo_meta::ObjectMeta::default(),
                        spec: Default::default(),
                        status: Default::default(),
                    }
                }
            }
        }
    } else {
        quote! {}
    } 
}

// Simple pluralizer.
// Duplicating the code from kube (without special casing) because it's simple enough.
// Irregular plurals must be explicitly specified.
fn to_plural(word: &str) -> String {
    // Words ending in s, x, z, ch, sh will be pluralized with -es (eg. foxes).
    if word.ends_with('s')
        || word.ends_with('x')
        || word.ends_with('z')
        || word.ends_with("ch")
        || word.ends_with("sh")
    {
        return format!("{word}es");
    }

    // Words ending in y that are preceded by a consonant will be pluralized by
    // replacing y with -ies (eg. puppies).
    if word.ends_with('y') {
        if let Some(c) = word.chars().nth(word.len() - 2) {
            if !matches!(c, 'a' | 'e' | 'i' | 'o' | 'u') {
                // Remove 'y' and add `ies`
                let mut chars = word.chars();
                chars.next_back();
                return format!("{}ies", chars.as_str());
            }
        }
    }

    // All other words will have "s" added to the end (eg. days).
    format!("{word}s")
}
