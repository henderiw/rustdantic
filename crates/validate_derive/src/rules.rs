use once_cell::sync::Lazy;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct ValidationRule {
    pub operator: String,
    pub value: Option<String>, // Optional because not all rules require a value (e.g., "required")
}

// Implement ToTokens for ValidationRule
impl ToTokens for ValidationRule {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let operator = &self.operator;
        let value = match &self.value {
            Some(val) => quote! { Some(#val.to_string()) },
            None => quote! { None },
        };

        tokens.extend(quote! {
            ValidationRule {
                operator: #operator.to_string(),
                value: #value,
            }
        });
    }
}

pub type ValidationHandler =
    fn(&ValidationRule, &proc_macro2::Ident, Option<String>, bool) -> TokenStream;

#[derive(Debug)]
pub struct RuleInfo {
    pub handler: ValidationHandler,
    pub supported_types: HashSet<&'static str>,
    pub option_only: bool,
    pub any_type: bool,
}

pub static RULE_REGISTRY: Lazy<HashMap<&'static str, RuleInfo>> = Lazy::new(|| {
    let mut registry: HashMap<&'static str, RuleInfo> = HashMap::new();

    // required rule
    registry.insert(
        "required",
        RuleInfo {
            handler: handle_required,
            supported_types: HashSet::new(),
            option_only: true,
            any_type: false,
        },
    );
    // numberic rules
    let operators = ["mo", "ge", "gt", "le", "lt"];
    for operator in &operators {
        registry.insert(
            *operator,
            RuleInfo {
                handler: handle_numeric_comparison,
                supported_types: {
                    let mut types = HashSet::new();
                    types.extend(["i32", "u32", "i64", "u64", "f32", "f64"]);
                    types
                },
                option_only: false,
                any_type: false,
            },
        );
    }

    // string rules
    let operators = ["maxLength", "minLength"];
    for operator in &operators {
        registry.insert(
            *operator,
            RuleInfo {
                handler: handle_length_comparison,
                supported_types: {
                    let mut types = HashSet::new();
                    types.extend(["String"]);
                    types
                },
                option_only: false,
                any_type: false,
            },
        );
    }

    registry.insert(
        "pattern",
        RuleInfo {
            handler: handle_pattern,
            supported_types: {
                let mut types = HashSet::new();
                types.extend(["String"]);
                types
            },
            option_only: false,
            any_type: false,
        },
    );
    

    // arrgys rules
    let operators = ["maxItems", "minItems"];
    for operator in &operators {
        registry.insert(
            *operator,
            RuleInfo {
                handler: handle_length_comparison,
                supported_types: {
                    let mut types = HashSet::new();
                    types.extend(["Vec", "HashMap", "BTreeMap"]);
                    types
                },
                option_only: false,
                any_type: false,
            },
        );
    }

    // custom rule
    registry.insert(
        "fn",
        RuleInfo {
            handler: handle_custom_function,
            supported_types: {
                let types = HashSet::new();
                types
            },
            option_only: false,
            any_type: true,
        },
    );
    registry
});

fn handle_required(
    _rule: &ValidationRule,
    field_name: &proc_macro2::Ident,
    _field_type: Option<String>,
    _is_option: bool,
) -> TokenStream {
    quote! {
        if self.#field_name.is_none() {
            errors.push(format!("Field '{}' is required", stringify!(#field_name)));
        }
    }
}

fn handle_custom_function(
    rule: &ValidationRule,
    field_name: &proc_macro2::Ident,
    _field_type: Option<String>,
    _is_option: bool,
) -> TokenStream {
    // Extract the custom function name from the rule
    if let Some(custom_fn_name) = &rule.value {
        let custom_fn_ident = syn::Ident::new(custom_fn_name, proc_macro2::Span::call_site());
        // If the field is an Option, validate only if it's `Some`
        quote! {
            if let Err(e) = self.#custom_fn_ident() {
                errors.push(format!(
                    "Field '{}' failed custom validation '{}': {}",
                    stringify!(#field_name),
                    stringify!(#custom_fn_name),
                    e
                ));
            }
        }
    } else {
        // Return a compile error if the function name is missing
        quote! {
            compile_error!(concat!(
                "Missing function name for custom validation rule on field `",
                stringify!(#field_name),
                "`."
            ));
        }
    }
}

fn handle_pattern(
    rule: &ValidationRule,
    field_name: &proc_macro2::Ident,
    _field_type: Option<String>,
    is_option: bool,
) -> TokenStream {
    match &rule.value {
        None => generate_compile_error("Missing threshold value", field_name),
        Some(val) => generate_pattern_code(field_name, val, is_option),
    }  
}


fn handle_length_comparison(
    rule: &ValidationRule,
    field_name: &proc_macro2::Ident,
    _field_type: Option<String>,
    is_option: bool,
) -> TokenStream {
    let threshold = if let Some(ref value_str) = rule.value {
        parse_threshold(value_str, Some("usize"), field_name)
    } else {
        return generate_compile_error("Missing threshold value", field_name);
    };

    let comparison_code = match rule.operator.as_str() {
        "minLength" => generate_length_comparison_code(field_name, &threshold, is_option, quote! { < }, quote! { >= }),
        "maxLength" => generate_length_comparison_code(field_name, &threshold, is_option, quote! { > }, quote! { <= }),
        "minItems" => generate_length_comparison_code(field_name, &threshold, is_option, quote! { < }, quote! { >= }),
        "maxItems" => generate_length_comparison_code(field_name, &threshold, is_option, quote! { > }, quote! { <= }),
        _ => {
            return generate_compile_error("Invalid operator", field_name);
        }
    };

    comparison_code
}

fn handle_numeric_comparison(
    rule: &ValidationRule,
    field_name: &proc_macro2::Ident,
    field_type: Option<String>,
    is_option: bool,
) -> TokenStream {
    let threshold = if let Some(ref value_str) = rule.value {
        parse_threshold(value_str, field_type.as_deref(), field_name)
    } else {
        return generate_compile_error("Missing threshold value", field_name);
    };

    let comparison_code = match rule.operator.as_str() {
        "ge" => generate_number_comparison_code(field_name, &threshold, is_option, quote! { < }, quote! { >= }),
        "gt" => generate_number_comparison_code(field_name, &threshold, is_option, quote! { <= }, quote! { > }),
        "le" => generate_number_comparison_code(field_name, &threshold, is_option, quote! { > }, quote! { <= }),
        "lt" => generate_number_comparison_code(field_name, &threshold, is_option, quote! { >= }, quote! { < }),
        "mo" => generate_modulo_code(field_name, &threshold, is_option),
        _ => {
            return generate_compile_error("Invalid operator", field_name);
        }
    };

    comparison_code
}

/// Generate comparison logic dynamically based on operator.
fn generate_length_comparison_code(
    field_name: &proc_macro2::Ident,
    threshold: &TokenStream,
    is_option: bool,
    invalid_op: TokenStream,
    valid_op: TokenStream,
) -> TokenStream {
    if is_option {
        quote! {
            if let Some(ref item) = self.#field_name {
                if item.len() #invalid_op #threshold {
                    errors.push(format!(
                        "Field '{}' length must be {} {}.",
                        stringify!(#field_name),
                        stringify!(#valid_op),
                        stringify!(#threshold),
                    ));
                }
            }
        }
    } else {
        quote! {
            if self.#field_name.len() #invalid_op #threshold {
                errors.push(format!(
                    "Field '{}' length must be {} {}.",
                    stringify!(#field_name),
                    stringify!(#valid_op),
                    stringify!(#threshold),
                ));
            }
        }
    }
}

/// Generate comparison logic dynamically based on operator.
fn generate_number_comparison_code(
    field_name: &proc_macro2::Ident,
    threshold: &TokenStream,
    is_option: bool,
    invalid_op: TokenStream,
    valid_op: TokenStream,
) -> TokenStream {
    if is_option {
        quote! {
            if let Some(item) = self.#field_name {
                if item #invalid_op #threshold {
                    errors.push(format!(
                        "Field '{}' must be {} {}.",
                        stringify!(#field_name),
                        stringify!(#valid_op),
                        stringify!(#threshold),
                    ));
                }
            }
        }
    } else {
        quote! {
            if self.#field_name #invalid_op #threshold {
                errors.push(format!(
                    "Field '{}' must be {} {}.",
                    stringify!(#field_name),
                    stringify!(#valid_op),
                    stringify!(#threshold),
                ));
            }
        }
    }
}


/// Generate modulo logic for "mo" operator.
fn generate_modulo_code(
    field_name: &proc_macro2::Ident,
    threshold: &TokenStream,
    is_option: bool,
) -> TokenStream {
    if is_option {
        quote! {
            if let Some(item) = self.#field_name {
                if item % #threshold != 0 {
                    errors.push(format!(
                        "Field '{}' must be a multiple of {}.",
                        stringify!(#field_name),
                        stringify!(#threshold),
                    ));
                }
            }
        }
    } else {
        quote! {
            if self.#field_name % #threshold != 0 {
                errors.push(format!(
                    "Field '{}' must be a multiple of {}.",
                    stringify!(#field_name),
                    stringify!(#threshold),
                ));
            }
        }
    }
}

fn generate_pattern_code(
    field_name: &proc_macro2::Ident,
    regex_pattern: &str,
    is_option: bool,
) -> TokenStream {
    if is_option {
        quote! {
            if let Some(ref item) = self.#field_name {
                let regex = regex::Regex::new(#regex_pattern).expect("Invalid regex pattern");
                if !regex.is_match(&item) {
                    errors.push(format!(
                        "Field '{}' does not match the required pattern: '{}'.",
                        stringify!(#field_name),
                        #regex_pattern,
                    ));
                }
            }
        }
    } else {
        quote! {
            let regex = regex::Regex::new(#regex_pattern).expect("Invalid regex pattern");
            if !regex.is_match(&self.#field_name) {
                errors.push(format!(
                    "Field '{}' does not match the required pattern: '{}'.",
                    stringify!(#field_name),
                    #regex_pattern,
                ));
            }
        }
    }
}

/// Parse the threshold value based on the field type.
fn parse_threshold(
    value_str: &str,
    field_type: Option<&str>,
    field_name: &proc_macro2::Ident,
) -> TokenStream {
    match field_type {
        Some("u32") => parse_threshold_value::<u32>(value_str, field_name, "u32"),
        Some("i32") => parse_threshold_value::<i32>(value_str, field_name, "i32"),
        Some("u64") => parse_threshold_value::<u64>(value_str, field_name, "u64"),
        Some("i64") => parse_threshold_value::<i64>(value_str, field_name, "i64"),
        Some("f32") | Some("f64") => parse_threshold_value::<f64>(value_str, field_name, "float"),
        Some("usize") => parse_threshold_value::<usize>(value_str, field_name, "usize"),
        _ => generate_compile_error("Unsupported field type", field_name),
    }
}


/// Helper to parse a threshold value for a specific type.
fn parse_threshold_value<T: std::str::FromStr + quote::ToTokens>(
    value_str: &str,
    field_name: &proc_macro2::Ident,
    _expected_type: &str,
) -> TokenStream {
    match value_str.parse::<T>() {
        Ok(val) => quote! { #val },
        Err(_) => generate_compile_error(value_str, field_name),
    }
}

/// Generate a compile-time error for invalid threshold values.
fn generate_compile_error(message: &str, field_name: &proc_macro2::Ident) -> TokenStream {
    quote! {
        compile_error!(concat!(
            #message,
            " for field `",
            stringify!(#field_name),
            "`."
        ));
    }
}