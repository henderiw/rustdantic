use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Attribute, DeriveInput, Field, Type, Data};
//use crate::enums::collect_all_enums;

pub(crate) fn derive(input: TokenStream) -> TokenStream {
    let derive_input: DeriveInput = match syn::parse2(input.clone()) {
        Err(err) => return err.to_compile_error(),
        Ok(di) => di,
    };

    // Limit derive to structs and inputs
    match derive_input.data {
        Data::Struct(_) | Data::Enum(_) => {}
        _ => {
            return syn::Error::new_spanned(
                &derive_input.ident,
                r#"Unions can not #[derive(ChoreoDefault)]"#,
            )
            .to_compile_error()
        }
    }

    let struct_name = &derive_input.ident;

    let defaults = match &derive_input.data {
        syn::Data::Struct(data_struct) => data_struct
            .fields
            .iter()
            .map(|field| generate_set_default_for_field(field))
            .collect::<Vec<_>>(),
        _ => vec![],
    };

    // this is the expanded code the compiler adds when the ChoreoDefault derive is added to a struct
    let expanded = quote! {
        impl ::choreo_api::Defaultable for #struct_name {
            /// Set defaults for all fields where applicable
            fn apply_defaults(&mut self) {
                #(#defaults)*
            }
        }
    };

    TokenStream::from(expanded)
}

fn generate_set_default_for_field(field: &Field) -> TokenStream {
    let field_name = field.ident.as_ref().expect("Expected named field");
    let field_type = &field.ty;
    let attr = extract_default_attribute(field);

    if is_type(field_type, "Option") {
        if let Some(inner_type) = extract_inner_type_for_type(field_type, "Option") {
            return generate_default_for_option(field_name, &inner_type, attr);
        }
    } else if is_type(field_type, "Vec") {
        if let Some(inner_type) = extract_inner_type_for_type(field_type, "Vec") {
            return generate_default_for_container(field_name, &inner_type, "Vec");
        }
    } else if is_type(field_type, "HashMap") {
        if let Some((_key, inner_type)) = extract_key_value_types_for_map(field_type, "HashMap") {
            return generate_default_for_container(field_name, &inner_type, "HashMap");
        }
    } else if is_type(field_type, "BTreeMap") {
        if let Some((_key, inner_type)) = extract_key_value_types_for_map(field_type, "BTreeMap") {
            return generate_default_for_container(field_name, &inner_type, "BTreeMap");
        }
    } else if is_nested_struct(field_type) {
        if field_name == "metadata" {
           return  quote! {} // Skip the `metadata` field
        } else {
            return quote! {
                self.#field_name.apply_defaults();
            }
        }
    }
    quote! {} // No action for unsupported types
}

fn generate_default_for_container(
    field_name: &proc_macro2::Ident,
    inner_type: &Type,
    container: &str,
) -> TokenStream {
    if is_nested_struct(inner_type) {
        if !is_owned_type(inner_type) {
            return quote! {
                compile_error!(concat!(container, " struct type is not owned"));
            };
        }
        return match container {
            "Vec" => quote! {
                for item in &mut self.#field_name {
                    item.apply_defaults();
                }
            },
            "HashMap" | "BTreeMap" => quote! {
                for (_key, value) in &mut self.#field_name {
                    value.apply_defaults();
                }
            },
            _ => quote! {},
        };
    }
    quote! {}
}

fn generate_default_for_option_container(
    field_name: &proc_macro2::Ident,
    inner_type: &Type,
    container: &str,
) -> TokenStream {
    if is_nested_struct(inner_type) {
        if !is_owned_type(inner_type) {
            return quote! {
                compile_error!(concat!("Optional ", container, " type is not owned"));
            };
        }
        return match container {
            "Vec" => quote! {
                if let Some(inner_vec) = self.#field_name.as_mut() {
                    for item in inner_vec {
                        item.apply_defaults();
                    }
                }
            },
            "HashMap" | "BTreeMap" => quote! {
                if let Some(inner_map) = self.#field_name.as_mut() {
                    for (_key, value) in inner_map {
                        value.apply_defaults();
                    }
                }
            },
            _ => quote! {},
        };
    }
    quote! {}
}

/// Extract the default attribute from the `#[default(...)]` if present.
///
/// Returns the default `Attribute` from the attributes, or `None` if the attribute is not present.
fn extract_default_attribute(field: &syn::Field) -> Option<&Attribute> {
    for attr in &field.attrs {
        // Check if the attribute is `default`
        if attr.path().is_ident("cdefault") {
            // Try to parse the attribute's argument as a general expression
            return Some(attr);
        }
    }
    None
}

fn generate_default_for_option(
    field_name: &proc_macro2::Ident,
    inner_type: &Type,
    attr: Option<&Attribute>,
) -> TokenStream {
    match attr {
        Some(attr) => {
            // default attribute is present
            let generated_code =
                generate_default_for_option_with_attribute(attr, field_name, inner_type);
            return generated_code;
        }
        None => {
            // no default attribute present
            if is_type(inner_type, "Vec") {
                if let Some(inner_type) = extract_inner_type_for_type(inner_type, "Vec") {
                    return generate_default_for_option_container(field_name, &inner_type, "Vec");
                }
            } else if is_type(inner_type, "HashMap") {
                if let Some((_key, inner_type)) =
                    extract_key_value_types_for_map(inner_type, "HashMap")
                {
                    return generate_default_for_option_container(
                        field_name,
                        &inner_type,
                        "HashMap",
                    );
                }
            } else if is_type(inner_type, "BTreeMap") {
                if let Some((_key, inner_type)) =
                    extract_key_value_types_for_map(inner_type, "BTreeMap")
                {
                    return generate_default_for_option_container(
                        field_name,
                        &inner_type,
                        "BTreeMap",
                    );
                }
            } else if is_nested_struct(inner_type) {
                return quote! {
                    if let Some(inner_item) = &mut self.#field_name {
                        inner_item.apply_defaults();
                    }
                };
            }
        }
    }
    quote! {} // No action if we cannot extract the inner type
}

fn generate_default_for_option_with_attribute(
    attr: &syn::Attribute,
    field_name: &proc_macro2::Ident,
    inner_type: &Type,
) -> TokenStream {
    match get_type_string(&inner_type).as_deref() {
        Some(type_name) if is_integer(type_name) => {
            if let Ok(lit_int) = attr.parse_args::<syn::LitInt>() {
                let value = lit_int.to_token_stream();
                return quote! {
                    if self.#field_name.is_none() {
                        self.#field_name = Some(#value);
                    }
                };
            }
        }
        Some(type_name) if is_float(type_name) => {
            if let Ok(lit_float) = attr.parse_args::<syn::LitFloat>() {
                let value = lit_float.to_token_stream();
                return quote! {
                    if self.#field_name.is_none() {
                        self.#field_name = Some(#value);
                    }
                };
            } else if let Ok(lit_int) = attr.parse_args::<syn::LitInt>() {
                let value = lit_int.base10_parse::<f64>().unwrap();
                return quote! {
                    if self.#field_name.is_none() {
                        self.#field_name = Some(#value);
                    }
                };
            }
        }
        Some(type_name) if is_string(type_name) => {
            if let Ok(lit_str) = attr.parse_args::<syn::LitStr>() {
                let value = lit_str.value();
                return quote! {
                    if self.#field_name.is_none() {
                        self.#field_name = Some(#value.to_string());
                    }
                };
            }
        }
        Some(type_name) if is_boolean(type_name) => {
            if let Ok(lit_bool) = attr.parse_args::<syn::LitBool>() {
                let value = lit_bool.value();
                return quote! {
                    if self.#field_name.is_none() {
                        self.#field_name = Some(#value);
                    }
                };
            }
        }
        Some(_) => {
            if let Ok(lit_str) = attr.parse_args::<syn::LitStr>() {
                let value = lit_str.value();
                if value.starts_with("enum=") {
                    let variant = value.trim_start_matches("enum=");
                    let variant_ident = syn::Ident::new(variant, proc_macro2::Span::call_site());
                    return quote! {
                        if self.#field_name.is_none() {
                            self.#field_name = Some(#inner_type::#variant_ident);
                        }
                    };
                }
                if value == "none" {
                    return quote! {};
                }
            }
        }
        None => {}
    }

    // Unified fallback for unsupported types or errors
    quote! {
        compile_error!(concat!(
            "Unsupported or invalid `#[cdefault(...)]` attribute format for field `",
            stringify!(#field_name),
            "`."
        ));
    }
}

fn is_nested_struct(typ: &syn::Type) -> bool {
    if let Some(type_name) = get_type_name(typ) {
        return !matches!(
            type_name.as_str(),
            "i8" | "i16"
                | "i32"
                | "i64"
                | "u8"
                | "u16"
                | "u32"
                | "u64"
                | "f32"
                | "f64"
                | "bool"
                | "String"
                | "Option"
                | "Vec"
                | "HashMap"
                | "BTreeMap"
        );
    }
    false
}

fn is_owned_type(typ: &syn::Type) -> bool {
    // Check if the type represents an owned struct
    if let syn::Type::Path(type_path) = typ {
        type_path
            .path
            .segments
            .iter()
            .all(|segment| segment.ident != "Ref" && segment.ident != "Borrowed")
    } else {
        false
    }
}

fn get_type_name(field_type: &Type) -> Option<String> {
    if let Type::Path(type_path) = field_type {
        if let Some(segment) = type_path.path.segments.last() {
            return Some(segment.ident.to_string());
        }
    }
    None
}

fn is_integer(typ: &str) -> bool {
    return matches!(
        typ,
        "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64"
    );
}

fn is_float(typ: &str) -> bool {
    return matches!(typ, "f32" | "f64");
}

fn is_string(typ: &str) -> bool {
    return typ == "String";
}

fn is_boolean(typ: &str) -> bool {
    return typ == "bool";
}

fn is_type(field_type: &Type, typ: &str) -> bool {
    if let Type::Path(type_path) = field_type {
        if let Some(segment) = type_path.path.segments.first() {
            return segment.ident == typ;
        }
    }
    false
}

fn get_type_string(typ: &syn::Type) -> Option<String> {
    // check if the input type us a syn::Type::Path -> primitive types like i32, String
    if let syn::Type::Path(type_path) = typ {
        // extract last identifier of the path -> e.g. i32 is the identifier
        if let Some(segment) = type_path.path.segments.last() {
            return Some(segment.ident.to_string());
        }
    }
    None
}

fn extract_inner_type_for_type(field_type: &Type, ident: &str) -> Option<Type> {
    if let Type::Path(type_path) = field_type {
        if let Some(segment) = type_path.path.segments.first() {
            if segment.ident == ident {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() {
                        return Some(inner_type.clone());
                    }
                }
            }
        }
    }
    None
}

fn extract_key_value_types_for_map(field_type: &Type, ident: &str) -> Option<(Type, Type)> {
    if let Type::Path(type_path) = field_type {
        if let Some(segment) = type_path.path.segments.first() {
            if segment.ident == ident {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    let mut types = args.args.iter().filter_map(|arg| {
                        if let syn::GenericArgument::Type(typ) = arg {
                            Some(typ.clone())
                        } else {
                            None
                        }
                    });
                    if let (Some(key_type), Some(value_type)) = (types.next(), types.next()) {
                        return Some((key_type, value_type));
                    }
                }
            }
        }
    }
    None
}
