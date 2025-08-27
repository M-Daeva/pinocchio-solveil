use {
    proc_macro::TokenStream,
    quote::quote,
    syn::{parse_macro_input, spanned::Spanned, Attribute, Data, DeriveInput, Fields, Meta},
};

/// Derive macro to generate TestAccounts struct from Accounts struct
///
/// Usage:
/// ```rust
/// #[derive(AccountMetas)]
/// pub struct Accounts<'a> {
///     #[account(signer, writable)]
///     pub sender: &'a AccountInfo,
///     #[account(writable)]
///     pub counter_account: &'a AccountInfo,
///     #[account]
///     pub system_program: &'a AccountInfo,
/// }
/// ```
#[proc_macro_derive(AccountMetas, attributes(account))]
pub fn derive_account_metas(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let test_name = syn::Ident::new(&format!("Test{}", name), name.span());

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return syn::Error::new_spanned(&input, "Only named fields are supported")
                    .to_compile_error()
                    .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(&input, "Only structs are supported")
                .to_compile_error()
                .into();
        }
    };

    let mut field_definitions = Vec::new();
    let mut meta_implementations = Vec::new();
    let mut info_implementations = Vec::new();

    for field in fields {
        let field_name = &field.ident;

        // Parse and validate account attributes
        let (is_signer, is_writable) = match parse_account_attributes(&field.attrs) {
            Ok(result) => result,
            Err(error) => return error.to_compile_error().into(),
        };

        field_definitions.push(quote! {
            pub #field_name: solana_pubkey::Pubkey
        });

        let meta_creation = if is_signer && is_writable {
            quote! {
                solana_instruction::AccountMeta::new(self.#field_name, true)
            }
        } else if is_signer && !is_writable {
            quote! {
                solana_instruction::AccountMeta::new_readonly(self.#field_name, true)
            }
        } else if !is_signer && is_writable {
            quote! {
                solana_instruction::AccountMeta::new(self.#field_name, false)
            }
        } else {
            quote! {
                solana_instruction::AccountMeta::new_readonly(self.#field_name, false)
            }
        };

        meta_implementations.push(meta_creation);
        info_implementations.push(quote! {
            (self.#field_name, #is_signer, #is_writable)
        });
    }

    let expanded = quote! {
        #[derive(Debug, Clone)]
        pub struct #test_name {
            #(#field_definitions,)*
        }

        impl #test_name {
            pub fn to_account_metas(&self) -> Vec<solana_instruction::AccountMeta> {
                vec![
                    #(#meta_implementations,)*
                ]
            }

            pub fn to_account_infos(&self) -> Vec<(solana_pubkey::Pubkey, bool, bool)> {
                vec![
                    #(#info_implementations,)*
                ]
            }
        }
    };

    TokenStream::from(expanded)
}

fn parse_account_attributes(attrs: &[Attribute]) -> Result<(bool, bool), syn::Error> {
    let mut is_signer = false;
    let mut is_writable = false;
    let mut errors = Vec::new();

    // Valid attribute values
    const VALID_ATTRS: &[&str] = &["signer", "writable"];

    for attr in attrs {
        if attr.path().is_ident("account") {
            match &attr.meta {
                Meta::List(meta_list) => {
                    // Parse comma-separated tokens inside #[account(...)]
                    let tokens = &meta_list.tokens;
                    let tokens_str = tokens.to_string();

                    for token in tokens_str.split(',') {
                        let token = token.trim();

                        if token.is_empty() {
                            continue;
                        }

                        match token {
                            "signer" => is_signer = true,
                            "writable" => is_writable = true,
                            _ => {
                                // Check for common typos and provide helpful suggestions
                                let suggestion = suggest_correction(token, VALID_ATTRS);
                                let error_msg = if let Some(suggestion) = suggestion {
                                    format!(
                                        "Unknown account attribute '{}'. Did you mean '{}'?",
                                        token, suggestion
                                    )
                                } else {
                                    format!(
                                        "Unknown account attribute '{}'. Valid attributes are: {}",
                                        token,
                                        VALID_ATTRS.join(", ")
                                    )
                                };

                                errors.push(syn::Error::new(attr.span(), error_msg));
                            }
                        }
                    }
                }
                Meta::Path(_) => {
                    // Just #[account] with no parameters - this is fine
                }
                Meta::NameValue(_) => {
                    errors.push(syn::Error::new(
                        attr.span(),
                        "account attribute does not support name-value syntax. Use #[account(signer, writable)] instead"
                    ));
                }
            }
        }
    }

    // Combine all errors into a single error
    if !errors.is_empty() {
        let combined_error = errors.into_iter().next().unwrap();
        // Note: In a real implementation, you might want to combine multiple errors
        // For now, we'll just return the first one
        return Err(combined_error);
    }

    Ok((is_signer, is_writable))
}

/// Simple string distance calculation for typo suggestions
fn suggest_correction<'a>(input: &'a str, valid_options: &'a [&'a str]) -> Option<&'a str> {
    let input_lower = input.to_lowercase();

    // First check for exact matches (case-insensitive)
    for &option in valid_options {
        if option.to_lowercase() == input_lower {
            return Some(option);
        }
    }

    // Then check for close matches using simple heuristics
    for &option in valid_options {
        if levenshtein_distance(&input_lower, &option.to_lowercase()) <= 2 {
            return Some(option);
        }
    }

    None
}

/// Simple Levenshtein distance implementation for typo detection
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.len();
    let len2 = s2.len();

    if len1 == 0 {
        return len2;
    }
    if len2 == 0 {
        return len1;
    }

    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    for i in 0..=len1 {
        matrix[i][0] = i;
    }
    for j in 0..=len2 {
        matrix[0][j] = j;
    }

    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();

    for i in 1..=len1 {
        for j in 1..=len2 {
            let cost = if s1_chars[i - 1] == s2_chars[j - 1] {
                0
            } else {
                1
            };
            matrix[i][j] = std::cmp::min(
                std::cmp::min(
                    matrix[i - 1][j] + 1, // deletion
                    matrix[i][j - 1] + 1, // insertion
                ),
                matrix[i - 1][j - 1] + cost, // substitution
            );
        }
    }

    matrix[len1][len2]
}
