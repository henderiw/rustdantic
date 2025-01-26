use crate::rules::{RuleInfo, ValidationRule, RULE_REGISTRY};
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::HashSet;
use syn::{DeriveInput, Field, Type, Data};

pub(crate) fn derive(input: TokenStream) -> TokenStream {
    let derive_input: DeriveInput = match syn::parse2(input.clone()) {
        Err(err) => return err.to_compile_error(),
        Ok(di) => di,
    };

    // Limit derive to structs
    match derive_input.data {
        Data::Struct(_) | Data::Enum(_) => {}
        _ => {
            return syn::Error::new_spanned(
                &derive_input.ident,
                r#"Unions can not #[derive(ChoreoValidate)]"#,
            )
            .to_compile_error()
        }
    }

    let struct_name = &derive_input.ident;

    let validations = match &derive_input.data {
        syn::Data::Struct(data_struct) => data_struct
            .fields
            .iter()
            .map(|field| generate_validations_for_field(field))
            .collect::<Vec<_>>(),
        _ => vec![],
    };

    // this is the expanded code the compiler adds when the ChoreoDefault derive is added to a struct
    let expanded = quote! {
        impl ::choreo_api::Validate for #struct_name {
            fn validate(&self) -> Result<(), String> {
                // errors collect the runtime validation errors.
                let mut errors: Vec<String> = Vec::new();
                #(#validations)*
                if errors.is_empty() {
                    Ok(())
                } else {
                    Err(errors.join("\n"))
                }
            }
        }
    };

    //eprintln!("Generated validation code {}", quote! { #expanded });

    TokenStream::from(expanded)
}

fn generate_validations_for_field(field: &Field) -> TokenStream {
    let field_name = field.ident.as_ref().expect("Expected named field");
    let field_type = &field.ty;
    match extract_validation_rules(field) {
        // Case 1: Rules successfully extracted
        Ok(rules) if !rules.is_empty() => {
            let field_validations: Vec<TokenStream> = rules
                .iter()
                .map(|rule| {

                    let operator = &rule.operator;
                    //eprintln!("Processing rule for field `{}`: operator = {}, value = {:?}", field_name, operator, rule.value);
                    if let Some(rule_info) = RULE_REGISTRY.get(rule.operator.as_str()) {
                        let (is_valid, field_type_str, is_option) =
                            is_type_valid(&field.ty, rule_info);

                        if !is_valid {
                            // Invalid rule for the field
                            return quote! {
                                compile_error!(concat!(
                                    "Unsupported or invalid `#[",
                                    stringify!(#operator),
                                    "]` attribute for field `",
                                    stringify!(#field_name),
                                    "`."
                                ));
                            };
                        }

                        // Call the handler to generate validation code
                        let c = (rule_info.handler)(rule, field_name, field_type_str, is_option);
                        //eprintln!("processing rule handler \n{}", c);
                        c
                    } else {
                        // Unknown rule
                        quote! {
                            compile_error!(concat!(
                                "Unknown validation rule: '",
                                stringify!(#operator),
                                "' for field `",
                                stringify!(#field_name),
                                "`."
                            ));
                        }
                    }
                })
                .collect();

            let c = quote! { #(#field_validations)* };
            //eprintln!("expanded code \n{}", c);
            c
        }

        // Case 2: Errors during rule extraction
        Err(err) => {
            quote! {
                compile_error!(#err);
            }
        }

        // Case 3: No rules provided for the field
        _ => {
            // No validation needed for this field
            if is_nested_struct(field_type) {
                if field_name == "metadata" {
                   return  quote! {} // Skip the `metadata` field
                } else {
                    return quote! {
                        if let Err(e) = &self.#field_name.validate() {
                            errors.push(format!(
                                "Field '{}' failed validation '{}'",
                                stringify!(#field_name),
                                e
                            ));
                        }
                    }
                }
            }
            quote! {}
        }
    }
}

/// Extract the `#[cvalidate(...)]` attributes from the field.
///
/// Returns `Ok(Vec<ValidationRule>)` if parsing succeeds, or `Err(String)` if duplicates or invalid rules are found.
fn extract_validation_rules(field: &syn::Field) -> Result<Vec<ValidationRule>, String> {
    let mut parsed_rules = Vec::new();
    let mut seen_rules = HashSet::new(); // To track duplicate operators

    for attr in &field.attrs {
        if attr.path().is_ident("cvalidate") {
            if let Ok(value) = attr.parse_args::<syn::LitStr>() {
                let rules = value.value();

                for rule in rules.split(',').map(|r| r.trim()) {
                    let parts: Vec<&str> = rule.split('=').collect();
                    let operator = parts[0].trim().to_string();
                    let value = parts.get(1).map(|&v| v.trim().to_string());

                    // Check for duplicates
                    if !seen_rules.insert(operator.clone()) {
                        let field_name = field
                            .ident
                            .as_ref()
                            .map_or("<unknown>".to_string(), |id| id.to_string());
                        return Err(format!(
                            "Duplicate validation rule `{}` found for field `{}`.",
                            operator,
                            &field_name // Pass a reference to `field_name`
                        ));
                    }

                    parsed_rules.push(ValidationRule { operator, value });
                }
            }
        }
    }

    Ok(parsed_rules)
}

/// Checks if the field type is valid based on the supported types.
/// Returns a tuple of (is_valid, type_name, is_option).
fn is_type_valid(field_type: &Type, rule_info: &RuleInfo) -> (bool, Option<String>, bool) {
    // Check if the field is an `Option`
    let (inner_type, is_option) = extract_type_and_option_status(field_type);
    let type_name = get_type_name(&inner_type);
    if rule_info.any_type {
        return (true, type_name, is_option);
    }
    if rule_info.option_only {
        if is_option {
            return (true, type_name, is_option);
        } else {
            return (false, type_name, is_option);
        }
    }
    let is_valid = rule_info
        .supported_types
        .contains(type_name.as_deref().unwrap_or(""));
    (is_valid, type_name, is_option)
}

/// Extracts type information, determining if the outer type is an `Option`
/// Returns a tuple `(Type, is_option)`:
/// - `Type`: The inner type if the outer type is `Option<T>`, or the original type if not.
/// - `is_option`: `true` if the outer type is an `Option`, otherwise `false`.
fn extract_type_and_option_status(field_type: &Type) -> (Type, bool) {
    if let syn::Type::Path(type_path) = field_type {
        if let Some(segment) = type_path.path.segments.first() {
            if segment.ident == "Option" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() {
                        return (inner_type.clone(), true);
                    }
                }
            }
        }
    }
    (field_type.clone(), false) // If not `Option`, return the original type and `false`
}

// Helper to get the type name
fn get_type_name(field_type: &Type) -> Option<String> {
    if let syn::Type::Path(type_path) = field_type {
        if let Some(segment) = type_path.path.segments.last() {
            return Some(segment.ident.to_string());
        }
    }
    None
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