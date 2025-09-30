use {
    proc_macro::TokenStream,
    quote::quote,
    syn::{parse::Parser, parse_macro_input, Data, DeriveInput, Field, Fields, Ident, Meta, Token},
};

/// A derive macro that generates flag management methods for structs with a flags field.
///
/// This macro generates constants and getter/setter methods for flag names specified
/// in the `#[optional(...)]` attribute on the `flags` field. The struct must have a
/// `flags` field of type `BitField`.
///
/// For flags that correspond to existing struct fields, it generates Option-based
/// getter and setter methods that manage both the flag and the field value.
///
/// For flags that don't correspond to existing fields, it generates flag getters/setters.
/// If a flag name starts with an underscore, it also generates Option-based
/// getter and setter methods treating it as a virtual boolean field.
///
/// # Example
///
/// ```rust
/// #[derive(OptionFlag)]
/// pub struct InstructionData {
///     #[optional(_is_paused, admin)]
///     pub flags: BitField,
///     pub admin: Pubkey,
/// }
/// ```
///
/// This generates:
/// - Constants: `_IS_PAUSED`, `IS_PAUSED`, `ADMIN`
/// - Flag getters/setters: `get_is_paused_flag()`, `set_is_paused_flag()`, etc.
/// - Field getters/setters: `get_is_paused()`, `set_is_paused()`, `get_admin()`, `set_admin()`
///
/// # Panics
///
/// - If applied to enums or unions
/// - If applied to structs without named fields
/// - If the struct doesn't have a `flags` field of type `BitField`
/// - If the `flags` field doesn't have an `#[optional(...)]` attribute
/// - If the `#[optional(...)]` attribute has invalid syntax
#[proc_macro_derive(OptionFlag, attributes(optional))]
pub fn derive_option_flag(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Validate that this is a struct with named fields
    let fields = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("OptionFlag can only be derived for structs with named fields"),
        },
        _ => panic!("OptionFlag can only be derived for structs"),
    };

    // Find the flags field and validate it
    let flags_field = find_and_validate_flags_field(fields);

    // Extract flag names from the #[optional(...)] attribute
    let flag_names = extract_flag_names(flags_field);

    // Create a map of field names to their types for lookup
    let field_map: std::collections::HashMap<String, &syn::Type> = fields
        .iter()
        .filter_map(|field| {
            field
                .ident
                .as_ref()
                .map(|ident| (ident.to_string(), &field.ty))
        })
        .collect();

    // Generate constants and methods
    let mut constants = Vec::new();
    let mut methods = Vec::new();
    let mut bit_index = 0u8;

    for flag_name in flag_names.iter() {
        let flag_name_str = flag_name.to_string();
        let is_underscore_prefixed = flag_name_str.starts_with('_');
        let clean_flag_name = if is_underscore_prefixed {
            Ident::new(&flag_name_str[1..], flag_name.span())
        } else {
            flag_name.clone()
        };

        // Determine bit indices and constant names
        let (const_ident, value_const_ident, next_bit_index) = if is_underscore_prefixed {
            let const_name = flag_name_str.to_uppercase();
            let const_ident = Ident::new(&const_name, flag_name.span());
            let value_const_name = flag_name_str[1..].to_uppercase();
            let value_const_ident = Ident::new(&value_const_name, flag_name.span());
            (const_ident, Some(value_const_ident), bit_index + 2)
        } else {
            let const_name = flag_name_str.to_uppercase();
            let const_ident = Ident::new(&const_name, flag_name.span());
            (const_ident, None, bit_index + 1)
        };

        // Generate constants
        let const_doc = format!("Flag bit index for the `{}` flag.", flag_name);
        constants.push(quote! {
            #[doc = #const_doc]
            const #const_ident: u8 = #bit_index;
        });

        if let Some(value_const_ident) = &value_const_ident {
            let value_const_doc = format!("Value bit index for the `{}` flag.", clean_flag_name);
            let value_index = bit_index + 1;
            constants.push(quote! {
                #[doc = #value_const_doc]
                const #value_const_ident: u8 = #value_index;
            });
        }

        // Generate flag getter method
        let getter_name = Ident::new(&format!("get_{}_flag", flag_name), flag_name.span());
        let getter_doc = format!("Returns whether the `{}` flag is set.", flag_name);
        let getter_visibility = if field_map.contains_key(&flag_name_str) || is_underscore_prefixed
        {
            quote! { fn } // private for existing fields or underscore-prefixed
        } else {
            quote! { pub fn } // public for standalone flags
        };
        let getter_non_snake_case = if is_underscore_prefixed {
            quote! { #[allow(non_snake_case)] }
        } else {
            quote! {}
        };

        methods.push(quote! {
            #[doc = #getter_doc]
            #getter_non_snake_case
            #[inline]
            #getter_visibility #getter_name(&self) -> bool {
                self.flags.get_flag(Self::#const_ident)
            }
        });

        // Generate flag setter method
        let setter_name = Ident::new(&format!("set_{}_flag", flag_name), flag_name.span());
        let setter_doc = format!("Sets the `{}` flag to the specified value.", flag_name);
        let setter_visibility = if field_map.contains_key(&flag_name_str) || is_underscore_prefixed
        {
            quote! { fn } // private for existing fields or underscore-prefixed
        } else {
            quote! { pub fn } // public for standalone flags
        };
        let setter_non_snake_case = if is_underscore_prefixed {
            quote! { #[allow(non_snake_case)] }
        } else {
            quote! {}
        };

        methods.push(quote! {
            #[doc = #setter_doc]
            #setter_non_snake_case
            #[inline]
            #setter_visibility #setter_name(&mut self, x: bool) {
                self.flags.set_flag(Self::#const_ident, x);
            }
        });

        // Handle existing field or underscore-prefixed flag
        if field_map.contains_key(&flag_name_str) {
            // Existing field: generate Option-based getter and setter
            let field_ident = Ident::new(&flag_name_str, flag_name.span());
            let field_type = field_map.get(&flag_name_str).unwrap();
            let option_getter_name = Ident::new(&format!("get_{}", flag_name), flag_name.span());
            let option_setter_name = Ident::new(&format!("set_{}", flag_name), flag_name.span());

            let option_getter_doc = format!(
                "Returns the value of `{}` if the flag is set, otherwise None.",
                flag_name
            );
            methods.push(quote! {
                #[doc = #option_getter_doc]
                #[inline]
                pub fn #option_getter_name(&self) -> Option<#field_type> {
                    if self.#getter_name() {
                        Some(self.#field_ident)
                    } else {
                        None
                    }
                }
            });

            let option_setter_doc = format!(
                "Sets the value of `{}` and updates the corresponding flag.",
                flag_name
            );
            methods.push(quote! {
                #[doc = #option_setter_doc]
                #[inline]
                pub fn #option_setter_name(&mut self, x: Option<#field_type>) {
                    match x {
                        Some(x) => {
                            self.#setter_name(true);
                            self.#field_ident = x;
                        },
                        None => {
                            self.#setter_name(false);
                        }
                    }
                }
            });
        } else if is_underscore_prefixed {
            // Underscore-prefixed flag: generate Option<bool> getter and setter
            let option_getter_name =
                Ident::new(&format!("get_{}", clean_flag_name), flag_name.span());
            let option_setter_name =
                Ident::new(&format!("set_{}", clean_flag_name), flag_name.span());

            let option_getter_doc = format!(
                "Returns the value of `{}` if the flag is set, otherwise None.",
                clean_flag_name
            );
            methods.push(quote! {
                #[doc = #option_getter_doc]
                #[inline]
                pub fn #option_getter_name(&self) -> Option<bool> {
                    if self.#getter_name() {
                        Some(self.flags.get_flag(Self::#value_const_ident))
                    } else {
                        None
                    }
                }
            });

            let option_setter_doc = format!(
                "Sets the value of `{}` and updates the corresponding flag.",
                clean_flag_name
            );
            methods.push(quote! {
                #[doc = #option_setter_doc]
                #[inline]
                pub fn #option_setter_name(&mut self, x: Option<bool>) {
                    match x {
                        Some(x) => {
                            self.#setter_name(true);
                            self.flags.set_flag(Self::#value_const_ident, x);
                        },
                        None => {
                            self.#setter_name(false);
                        }
                    }
                }
            });
        }

        bit_index = next_bit_index;
    }

    // Generate the impl block
    let expanded = quote! {
        impl #name {
            #(#constants)*
            #(#methods)*
        }
    };

    TokenStream::from(expanded)
}

/// Finds the flags field and validates that it exists and has the correct type.
///
/// # Panics
///
/// Panics if:
/// - No 'flags' field is found
/// - The 'flags' field is not of type BitField
fn find_and_validate_flags_field(fields: &syn::punctuated::Punctuated<Field, Token![,]>) -> &Field {
    let flags_field = fields.iter().find(|field| {
        field
            .ident
            .as_ref()
            .map(|ident| ident == "flags")
            .unwrap_or(false)
    });

    let flags_field = match flags_field {
        Some(field) => field,
        None => panic!("OptionFlag requires a 'flags' field of type BitField"),
    };

    // Check if the field type is BitField
    if !is_bitfield_type(&flags_field.ty) {
        panic!("The 'flags' field must be of type BitField");
    }

    flags_field
}

/// Extracts flag names from the #[optional(...)] attribute on the flags field.
///
/// # Panics
///
/// Panics if:
/// - The flags field doesn't have an #[optional(...)] attribute
/// - The attribute has invalid syntax
fn extract_flag_names(flags_field: &Field) -> Vec<Ident> {
    for attr in &flags_field.attrs {
        if attr.path().is_ident("optional") {
            // Parse the attribute arguments
            match &attr.meta {
                Meta::List(meta_list) => {
                    // Parse the tokens inside the parentheses
                    let tokens = &meta_list.tokens;

                    // Convert TokenStream to a parseable format
                    let parser = syn::punctuated::Punctuated::<Ident, Token![,]>::parse_terminated;
                    match parser.parse2(tokens.clone()) {
                        Ok(parsed_idents) => {
                            // Convert to owned Vec<Ident>
                            return parsed_idents.into_iter().collect();
                        }
                        Err(e) => panic!(
                            "Failed to parse #[optional(...)] attribute arguments: {}. \
                             Expected comma-separated identifiers like #[optional(flag1, flag2)]",
                            e
                        ),
                    }
                }
                _ => panic!(
                    "#[optional] attribute on flags field requires parentheses with flag names. \
                     Use #[optional(flag1, flag2, ...)]"
                ),
            }
        }
    }

    panic!(
        "The 'flags' field must have an #[optional(...)] attribute specifying the flag names. \
         Use #[optional(flag1, flag2, ...)]"
    );
}

/// Checks if a type is BitField.
///
/// This function handles simple cases like `BitField` and module-qualified paths
/// like `some_module::BitField`.
fn is_bitfield_type(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Path(syn::TypePath { path, .. }) => {
            // Check if the last segment is "BitField"
            path.segments
                .last()
                .map(|segment| segment.ident == "BitField")
                .unwrap_or(false)
        }
        _ => false,
    }
}
