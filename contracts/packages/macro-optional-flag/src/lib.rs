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
/// For flags that correspond to existing struct fields, it also generates Option-based
/// getter and setter methods that manage both the flag and the field value.
///
/// # Example
///
/// ```rust
/// #[derive(OptionFlag)]
/// pub struct UserId {
///     #[optional(is_open, account_bump)]
///     pub flags: BitField,
///     pub id: Uint32,
///     pub account_bump: u8,
///     pub rotation_state_bump: u8,
/// }
/// ```
///
/// This generates:
/// - Constants: `IS_OPEN`, `ACCOUNT_BUMP`
/// - Flag getters/setters: `get_is_open_flag()`, `set_is_open_flag()`, etc.
/// - Field getters/setters (for existing fields): `get_account_bump()`, `set_account_bump()`
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

    for (index, flag_name) in flag_names.iter().enumerate() {
        let const_name = flag_name.to_string().to_uppercase();
        let const_ident = Ident::new(&const_name, flag_name.span());
        let getter_name = Ident::new(&format!("get_{}_flag", flag_name), flag_name.span());
        let setter_name = Ident::new(&format!("set_{}_flag", flag_name), flag_name.span());

        // Generate constant with documentation
        let index_u8 = index as u8;
        let const_doc = format!("Flag bit index for the `{}` flag.", flag_name);
        constants.push(quote! {
            #[doc = #const_doc]
            const #const_ident: u8 = #index_u8;
        });

        // Check if this flag corresponds to an existing field
        let flag_name_str = flag_name.to_string();
        let has_corresponding_field = field_map.contains_key(&flag_name_str);

        // Generate flag getter method with appropriate visibility
        let getter_doc = format!("Returns whether the `{}` flag is set.", flag_name);
        let getter_visibility = if has_corresponding_field {
            quote! { fn } // private
        } else {
            quote! { pub fn } // public
        };

        methods.push(quote! {
            #[doc = #getter_doc]
            #[inline]
            #getter_visibility #getter_name(&self) -> bool {
                self.flags.get_flag(Self::#const_ident)
            }
        });

        // Generate flag setter method with appropriate visibility
        let setter_doc = format!("Sets the `{}` flag to the specified value.", flag_name);
        let setter_visibility = if has_corresponding_field {
            quote! { fn } // private
        } else {
            quote! { pub fn } // public
        };

        methods.push(quote! {
            #[doc = #setter_doc]
            #[inline]
            #setter_visibility #setter_name(&mut self, x: bool) {
                self.flags.set_flag(Self::#const_ident, x);
            }
        });

        // If there's a corresponding field, generate Option-based getter and setter
        if let Some(field_type) = field_map.get(&flag_name_str) {
            let field_ident = Ident::new(&flag_name_str, flag_name.span());
            let option_getter_name = Ident::new(&format!("get_{}", flag_name), flag_name.span());
            let option_setter_name = Ident::new(&format!("set_{}", flag_name), flag_name.span());

            // Generate Option-based getter
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

            // Generate Option-based setter
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
        }
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
