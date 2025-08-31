use {
    proc_macro::TokenStream,
    quote::quote,
    syn::{parse_macro_input, Data, DeriveInput, Fields, Type},
};

/// Derive macro for ZeroCopySerialize trait
#[proc_macro_derive(ZCSerialize)]
pub fn derive_zc_serialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let serialize_fields = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields) => {
                let field_serializers = fields.named.iter().map(|field| {
                    let field_name = &field.ident;
                    let field_type = &field.ty;

                    generate_serialize_field_code(field_name.as_ref().unwrap(), field_type)
                });

                quote! {
                    ByteWriter::new(data)
                        #(.#field_serializers)*
                        .complete()
                }
            }
            _ => panic!("ZCSerialize only supports structs with named fields"),
        },
        _ => panic!("ZCSerialize only supports structs"),
    };

    let expanded = quote! {
        impl ZeroCopySerialize for #name {
            fn serialize_into(&self, data: &mut [u8]) -> ProgramResult {
                #serialize_fields
            }
        }
    };

    TokenStream::from(expanded)
}

/// Derive macro for ZeroCopyDeserialize trait
#[proc_macro_derive(ZCDeserialize)]
pub fn derive_zc_deserialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let deserialize_fields = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields) => {
                let field_deserializers = fields.named.iter().map(|field| {
                    let field_name = &field.ident;
                    let field_type = &field.ty;

                    generate_deserialize_field_code(field_name.as_ref().unwrap(), field_type)
                });

                quote! {
                    ByteReader::new::<Self>(data, start_index)
                        #(.#field_deserializers)*
                        .complete()
                }
            }
            _ => panic!("ZCDeserialize only supports structs with named fields"),
        },
        _ => panic!("ZCDeserialize only supports structs"),
    };

    let expanded = quote! {
        impl ZeroCopyDeserialize for #name {
            fn deserialize_from(data: &[u8], start_index: usize) -> Result<(Self, usize)> {
                #deserialize_fields
            }
        }
    };

    TokenStream::from(expanded)
}

/// Generate serialization code for a specific field based on its type
fn generate_serialize_field_code(
    field_name: &syn::Ident,
    field_type: &Type,
) -> proc_macro2::TokenStream {
    match field_type {
        // Handle basic types
        Type::Path(type_path) if type_path.path.segments.len() == 1 => {
            let segment = &type_path.path.segments[0];
            let type_name = segment.ident.to_string();

            match type_name.as_str() {
                "bool" => quote! { write_bool(self.#field_name) },
                "u8" => quote! { write_u8(self.#field_name) },
                "u16" => quote! { write_u16(self.#field_name) },
                "u32" => quote! { write_u32(self.#field_name) },
                "u64" => quote! { write_u64(self.#field_name) },
                "u128" => quote! { write_u128(self.#field_name) },
                "String" => quote! { write_string(&self.#field_name) },
                "Pubkey" => quote! { write_pubkey(&self.#field_name) },
                _ => {
                    // For custom types, assume they implement ZeroCopySerialize
                    quote! { write_custom(&self.#field_name) }
                }
            }
        }
        // Handle Option<T>
        Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.first() {
                if segment.ident == "Option" {
                    // Extract the inner type from Option<T>
                    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                        if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() {
                            return generate_option_serialize_code(field_name, inner_type);
                        }
                    }
                }
            }
            // Default case for complex types
            quote! { write_custom(&self.#field_name) }
        }
        _ => {
            // Default case for any other complex types
            quote! { write_custom(&self.#field_name) }
        }
    }
}

/// Generate deserialization code for a specific field based on its type
fn generate_deserialize_field_code(
    field_name: &syn::Ident,
    field_type: &Type,
) -> proc_macro2::TokenStream {
    match field_type {
        // Handle basic types
        Type::Path(type_path) if type_path.path.segments.len() == 1 => {
            let segment = &type_path.path.segments[0];
            let type_name = segment.ident.to_string();

            match type_name.as_str() {
                "bool" => quote! { read_bool(|x| &mut x.#field_name) },
                "u8" => quote! { read_u8(|x| &mut x.#field_name) },
                "u16" => quote! { read_u16(|x| &mut x.#field_name) },
                "u32" => quote! { read_u32(|x| &mut x.#field_name) },
                "u64" => quote! { read_u64(|x| &mut x.#field_name) },
                "u128" => quote! { read_u128(|x| &mut x.#field_name) },
                "String" => quote! { read_string(|x| &mut x.#field_name) },
                "Pubkey" => quote! { read_pubkey(|x| &mut x.#field_name) },
                _ => {
                    // For custom types, assume they implement ZeroCopyDeserialize
                    quote! { read_custom(|x| &mut x.#field_name) }
                }
            }
        }
        // Handle Option<T>
        Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.first() {
                if segment.ident == "Option" {
                    // Extract the inner type from Option<T>
                    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                        if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() {
                            return generate_option_deserialize_code(field_name, inner_type);
                        }
                    }
                }
            }
            // Default case for complex types
            quote! { read_custom(|x| &mut x.#field_name) }
        }
        _ => {
            // Default case for any other complex types
            quote! { read_custom(|x| &mut x.#field_name) }
        }
    }
}

/// Generate serialization code for Option<T> fields
fn generate_option_serialize_code(
    field_name: &syn::Ident,
    inner_type: &Type,
) -> proc_macro2::TokenStream {
    match inner_type {
        Type::Path(type_path) if type_path.path.segments.len() == 1 => {
            let segment = &type_path.path.segments[0];
            let type_name = segment.ident.to_string();

            match type_name.as_str() {
                "Pubkey" => quote! {
                    write_bytes(&option_as_bytes(&self.#field_name, pubkey_as_bytes))
                },
                "bool" => quote! {
                    write_bytes(&option_as_bytes(&self.#field_name, bool_as_bytes))
                },
                "u8" => quote! {
                    write_bytes(&option_as_bytes(&self.#field_name, u8_as_bytes))
                },
                "u16" => quote! {
                    write_bytes(&option_as_bytes(&self.#field_name, u16_as_bytes))
                },
                "u32" => quote! {
                    write_bytes(&option_as_bytes(&self.#field_name, u32_as_bytes))
                },
                "u64" => quote! {
                    write_bytes(&option_as_bytes(&self.#field_name, u64_as_bytes))
                },
                "u128" => quote! {
                    write_bytes(&option_as_bytes(&self.#field_name, u128_as_bytes))
                },
                _ => {
                    // For custom types in Option
                    quote! { write_option_custom(&self.#field_name) }
                }
            }
        }
        _ => {
            // For complex types in Option
            quote! { write_option_custom(&self.#field_name) }
        }
    }
}

/// Generate deserialization code for Option<T> fields
fn generate_option_deserialize_code(
    field_name: &syn::Ident,
    inner_type: &Type,
) -> proc_macro2::TokenStream {
    match inner_type {
        Type::Path(type_path) if type_path.path.segments.len() == 1 => {
            let segment = &type_path.path.segments[0];
            let type_name = segment.ident.to_string();

            match type_name.as_str() {
                "Pubkey" => quote! {
                    read_option(|x| &mut x.#field_name, to_pubkey)
                },
                "bool" => quote! {
                    read_option(|x| &mut x.#field_name, to_bool)
                },
                "u8" => quote! {
                    read_option(|x| &mut x.#field_name, to_u8)
                },
                "u16" => quote! {
                    read_option(|x| &mut x.#field_name, to_u16)
                },
                "u32" => quote! {
                    read_option(|x| &mut x.#field_name, to_u32)
                },
                "u64" => quote! {
                    read_option(|x| &mut x.#field_name, to_u64)
                },
                "u128" => quote! {
                    read_option(|x| &mut x.#field_name, to_u128)
                },
                "String" => quote! {
                    read_option(|x| &mut x.#field_name, to_string)
                },
                _ => {
                    // For custom types in Option, we need to create a parser function
                    quote! {
                        read_option(|x| &mut x.#field_name, |data, pos| {
                            let (value, new_pos) = <#inner_type>::deserialize_from(data, pos)?;
                            Ok((value, new_pos))
                        })
                    }
                }
            }
        }
        _ => {
            // For complex types in Option
            quote! {
                read_option(|x| &mut x.#field_name, |data, pos| {
                    let (value, new_pos) = <#inner_type>::deserialize_from(data, pos)?;
                    Ok((value, new_pos))
                })
            }
        }
    }
}
