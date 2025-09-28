use {
    proc_macro::TokenStream,
    quote::quote,
    syn::{
        parse_macro_input, punctuated::Punctuated, token::Comma, Attribute, Data, DeriveInput,
        Field, Fields, Ident, Meta, Path,
    },
};

/// Generates an enum and wrapper implementation from a struct with #[enumfields(...)] attribute
///
/// Usage:
/// ```
/// #[derive(CodamaType, EnumWrapper)]
/// #[p_serde]
/// pub struct TargetEnum(#[enumfields(spl, proxy, route)] u8);
/// ```
///
/// This will generate:
/// - An enum with the specified variants
/// - Implementation methods for the wrapper struct
#[proc_macro_derive(EnumWrapper, attributes(enumfields))]
pub fn derive_enum_wrapper(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let wrapper_name = &input.ident;

    // Extract enum name by removing "Enum" suffix if present
    let enum_name_str = wrapper_name.to_string();
    let enum_name = if enum_name_str.ends_with("Enum") {
        Ident::new(
            &enum_name_str[..enum_name_str.len() - 4],
            wrapper_name.span(),
        )
    } else {
        return syn::Error::new_spanned(
            wrapper_name,
            "Struct name should end with 'Enum' for EnumWrapper derivation",
        )
        .to_compile_error()
        .into();
    };

    // Extract the underlying type and enumfields attribute from the struct field
    let (underlying_type, enum_variants) = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                let field = fields.unnamed.first().unwrap();
                let underlying_type = &field.ty;
                match extract_enum_variants(field) {
                    Ok(variants) => (underlying_type, variants),
                    Err(e) => return e.to_compile_error().into(),
                }
            }
            _ => {
                return syn::Error::new_spanned(
                    &input,
                    "EnumWrapper requires a tuple struct with exactly one field",
                )
                .to_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(
                &input,
                "EnumWrapper can only be used on tuple structs",
            )
            .to_compile_error()
            .into();
        }
    };

    // Generate enum variants
    let enum_variant_definitions = enum_variants
        .iter()
        .enumerate()
        .map(|(index, variant_name)| {
            if index == 0 {
                quote! {
                    #[default]
                    #variant_name,
                }
            } else {
                quote! {
                    #variant_name,
                }
            }
        });

    // Generate match arms for the get() method
    let match_arms = enum_variants
        .iter()
        .enumerate()
        .map(|(index, variant_name)| {
            let index_lit = proc_macro2::Literal::usize_unsuffixed(index);
            quote! {
                #index_lit => #enum_name::#variant_name,
            }
        });

    // Extract derive attributes from the original struct, excluding EnumWrapper
    let filtered_derives = filter_derive_attributes(&input.attrs);

    let expanded = quote! {
        #(#filtered_derives)*
        #[derive(Default, Debug, PartialEq)]
        #[repr(#underlying_type)]
        pub enum #enum_name {
            #(#enum_variant_definitions)*
        }

        impl From<#enum_name> for #wrapper_name {
            fn from(x: #enum_name) -> Self {
                Self(x as #underlying_type)
            }
        }

        impl #wrapper_name {
            #[inline]
            pub fn get_raw(&self) -> #underlying_type {
                self.0
            }

            #[inline]
            pub fn set_raw(&mut self, x: #underlying_type) {
                self.0 = x;
            }

            #[inline]
            pub fn get(&self) -> #enum_name {
                match self.0 {
                    #(#match_arms)*
                    _ => #enum_name::default(),
                }
            }

            #[inline]
            pub fn set(&mut self, x: #enum_name) {
                self.0 = x as #underlying_type;
            }
        }
    };

    TokenStream::from(expanded)
}

fn extract_enum_variants(field: &Field) -> Result<Vec<Ident>, syn::Error> {
    for attr in &field.attrs {
        if attr.path().is_ident("enumfields") {
            return parse_enumfields_attribute(attr);
        }
    }

    Err(syn::Error::new_spanned(
        field,
        "Field must have #[enumfields(...)] attribute with variant names",
    ))
}

fn parse_enumfields_attribute(attr: &Attribute) -> Result<Vec<Ident>, syn::Error> {
    match &attr.meta {
        Meta::List(meta_list) => {
            let paths: Result<Punctuated<Path, Comma>, _> =
                meta_list.parse_args_with(Punctuated::parse_terminated);

            match paths {
                Ok(paths) => {
                    let mut variants = Vec::new();

                    for path in paths {
                        if let Some(ident) = path.get_ident() {
                            // Convert to PascalCase
                            let pascal_case = to_pascal_case(&ident.to_string());
                            variants.push(Ident::new(&pascal_case, ident.span()));
                        } else {
                            return Err(syn::Error::new_spanned(
                                path,
                                "Expected simple identifier in enumfields",
                            ));
                        }
                    }

                    Ok(variants)
                }
                Err(e) => Err(syn::Error::new_spanned(
                    attr,
                    format!("Failed to parse enumfields arguments: {}", e),
                )),
            }
        }
        _ => Err(syn::Error::new_spanned(
            attr,
            "enumfields attribute must be a list: #[enumfields(variant1, variant2, ...)]",
        )),
    }
}

fn to_pascal_case(s: &str) -> String {
    s.chars()
        .next()
        .map(|c| c.to_uppercase().collect::<String>())
        .unwrap_or_default()
        + &s[1..]
}

fn filter_derive_attributes(attrs: &[Attribute]) -> Vec<proc_macro2::TokenStream> {
    attrs
        .iter()
        .filter_map(|attr| {
            if attr.path().is_ident("derive") {
                // Parse the derive attribute and filter out EnumWrapper
                if let Meta::List(meta_list) = &attr.meta {
                    if let Ok(paths) =
                        meta_list.parse_args_with(Punctuated::<Path, Comma>::parse_terminated)
                    {
                        let filtered_derives: Vec<_> = paths
                            .into_iter()
                            .filter(|path| !path.is_ident("EnumWrapper"))
                            .collect();

                        if !filtered_derives.is_empty() {
                            return Some(quote! {
                                #[derive(#(#filtered_derives),*)]
                            });
                        }
                    }
                }
            }
            None
        })
        .collect()
}
