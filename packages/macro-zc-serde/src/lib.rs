use {
    proc_macro::TokenStream,
    quote::quote,
    syn::{parse_macro_input, Data, DeriveInput, Fields, Type, TypePath},
};

/// Derive macro for automatically implementing ZeroCopySerialize
#[proc_macro_derive(ZCSerialize)]
pub fn derive_zero_copy_serialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let serialize_body = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields) => {
                let field_writes = fields.named.iter().map(|field| {
                    let field_name = &field.ident;
                    let field_type = &field.ty;

                    generate_write_call(field_name.as_ref().unwrap(), field_type)
                });

                quote! {
                    ByteWriter::new(data)
                        #(#field_writes)*
                        .complete()
                }
            }
            Fields::Unnamed(_) => {
                panic!("ZCSerialize can only be derived for structs with named fields");
            }
            Fields::Unit => {
                quote! {
                    ByteWriter::new(data).complete()
                }
            }
        },
        _ => panic!("ZCSerialize can only be derived for structs"),
    };

    let expanded = quote! {
        impl #impl_generics ZeroCopySerialize for #name #ty_generics #where_clause {
            fn serialize_into(&self, data: &mut [u8]) -> ProgramResult {
                #serialize_body
            }
        }
    };

    TokenStream::from(expanded)
}

fn generate_write_call(field_name: &syn::Ident, field_type: &Type) -> proc_macro2::TokenStream {
    match field_type {
        Type::Path(TypePath { path, .. }) => {
            let type_name = path.segments.last().unwrap().ident.to_string();

            match type_name.as_str() {
                "u8" => quote! { .write_u8(self.#field_name)? },
                "u16" => quote! { .write_u16(self.#field_name)? },
                "u32" => quote! { .write_u32(self.#field_name)? },
                "u64" => quote! { .write_u64(self.#field_name)? },
                "u128" => quote! { .write_u128(self.#field_name)? },
                "bool" => quote! { .write_bool(self.#field_name)? },
                "String" => quote! { .write_string(&self.#field_name)? },
                "Pubkey" => quote! { .write_pubkey(&self.#field_name)? },
                _ => {
                    // Check if it's an Option type
                    if let Some(segment) = path.segments.last() {
                        if segment.ident == "Option" {
                            if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                                if let Some(syn::GenericArgument::Type(inner_type)) =
                                    args.args.first()
                                {
                                    return generate_option_write_call(field_name, inner_type);
                                }
                            }
                        }
                    }

                    // For custom types that implement ZeroCopySerialize
                    quote! { .write_custom(&self.#field_name)? }
                }
            }
        }
        _ => {
            // For any other complex types, assume they implement ZeroCopySerialize
            quote! { .write_custom(&self.#field_name)? }
        }
    }
}

fn generate_option_write_call(
    field_name: &syn::Ident,
    inner_type: &Type,
) -> proc_macro2::TokenStream {
    match inner_type {
        Type::Path(TypePath { path, .. }) => {
            let type_name = path.segments.last().unwrap().ident.to_string();

            match type_name.as_str() {
                "Pubkey" => {
                    quote! { .write_option(&self.#field_name, |v| v.as_ref())? }
                }
                "String" => {
                    quote! { .write_option(&self.#field_name, |v| v.as_bytes())? }
                }
                // For primitive types, use a different approach that avoids temporary references
                "u8" => {
                    quote! {
                        .write_option_primitive(&self.#field_name, |writer, v| writer.write_u8(*v))?
                    }
                }
                "u16" => {
                    quote! {
                        .write_option_primitive(&self.#field_name, |writer, v| writer.write_u16(*v))?
                    }
                }
                "u32" => {
                    quote! {
                        .write_option_primitive(&self.#field_name, |writer, v| writer.write_u32(*v))?
                    }
                }
                "u64" => {
                    quote! {
                        .write_option_primitive(&self.#field_name, |writer, v| writer.write_u64(*v))?
                    }
                }
                "u128" => {
                    quote! {
                        .write_option_primitive(&self.#field_name, |writer, v| writer.write_u128(*v))?
                    }
                }
                "bool" => {
                    quote! {
                        .write_option_primitive(&self.#field_name, |writer, v| writer.write_bool(*v))?
                    }
                }
                _ => {
                    // For custom types that implement ZeroCopySerialize + ZeroCopyDeserialize
                    quote! { .write_option_custom(&self.#field_name)? }
                }
            }
        }
        _ => {
            // For complex Option types, use write_option_custom
            quote! { .write_option_custom(&self.#field_name)? }
        }
    }
}

/// Derive macro for automatically implementing ZeroCopyDeserialize
#[proc_macro_derive(ZCDeserialize)]
pub fn derive_zc_deserialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let read_chain = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields) => {
                let field_reads = fields.named.iter().map(|field| {
                    let field_name = &field.ident;
                    let field_type = &field.ty;

                    generate_read_call(field_name.as_ref().unwrap(), field_type)
                });

                quote! {
                    ByteReader::new::<Self>(data, start_index)
                        #(#field_reads)*
                        .complete()
                }
            }
            Fields::Unnamed(_) => {
                panic!("ZCDeserialize can only be derived for structs with named fields");
            }
            Fields::Unit => {
                quote! {
                    ByteReader::new::<Self>(data, start_index).complete()
                }
            }
        },
        _ => panic!("ZCDeserialize can only be derived for structs"),
    };

    let expanded = quote! {
        impl #impl_generics ZeroCopyDeserialize for #name #ty_generics #where_clause {
            fn deserialize_from(data: &[u8], start_index: usize) -> Result<(Self, usize)> {
                #read_chain
            }
        }
    };

    TokenStream::from(expanded)
}

/// Generate the appropriate read call based on field type
fn generate_read_call(field_name: &syn::Ident, field_type: &Type) -> proc_macro2::TokenStream {
    match field_type {
        // Handle basic types
        Type::Path(type_path) => {
            let type_name = &type_path.path.segments.last().unwrap().ident;

            match type_name.to_string().as_str() {
                "bool" => quote! {
                    .read_bool(|x| &mut x.#field_name)?
                },
                "u8" => quote! {
                    .read_u8(|x| &mut x.#field_name)?
                },
                "u16" => quote! {
                    .read_u16(|x| &mut x.#field_name)?
                },
                "u32" => quote! {
                    .read_u32(|x| &mut x.#field_name)?
                },
                "u64" => quote! {
                    .read_u64(|x| &mut x.#field_name)?
                },
                "u128" => quote! {
                    .read_u128(|x| &mut x.#field_name)?
                },
                "Pubkey" => quote! {
                    .read_pubkey(|x| &mut x.#field_name)?
                },
                "String" => quote! {
                    .read_string(|x| &mut x.#field_name)?
                },
                "Option" => {
                    // Handle Option<T>
                    if let syn::PathArguments::AngleBracketed(args) =
                        &type_path.path.segments.last().unwrap().arguments
                    {
                        if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() {
                            let parser_fn = get_parser_function(inner_type);
                            quote! {
                                .read_option(|x| &mut x.#field_name, #parser_fn)?
                            }
                        } else {
                            panic!("Option type must have a type parameter");
                        }
                    } else {
                        panic!("Option type must have angle bracketed arguments");
                    }
                }
                "Vec" => {
                    // Handle Vec<T>
                    if let syn::PathArguments::AngleBracketed(args) =
                        &type_path.path.segments.last().unwrap().arguments
                    {
                        if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() {
                            let parser_fn = get_parser_function(inner_type);
                            quote! {
                                .read_vec_fixed(|x| &mut x.#field_name, #parser_fn)?
                            }
                        } else {
                            panic!("Vec type must have a type parameter");
                        }
                    } else {
                        panic!("Vec type must have angle bracketed arguments");
                    }
                }
                _ => {
                    // For custom types that implement ZeroCopyDeserialize
                    quote! {
                        .read_custom(|x| &mut x.#field_name)?
                    }
                }
            }
        }
        // Handle array types [T; N]
        Type::Array(array_type) => {
            let parser_fn = get_parser_function(&array_type.elem);
            quote! {
                .read_array(|x| &mut x.#field_name, #parser_fn)?
            }
        }
        _ => {
            // Fallback to custom for unknown types
            quote! {
                .read_custom(|x| &mut x.#field_name)?
            }
        }
    }
}

/// Get the appropriate parser function for a given type
fn get_parser_function(ty: &Type) -> proc_macro2::TokenStream {
    match ty {
        Type::Path(type_path) => {
            let type_name = &type_path.path.segments.last().unwrap().ident;

            match type_name.to_string().as_str() {
                "bool" => quote! { to_bool },
                "u8" => quote! { to_u8 },
                "u16" => quote! { to_u16 },
                "u32" => quote! { to_u32 },
                "u64" => quote! { to_u64 },
                "u128" => quote! { to_u128 },
                "Pubkey" => quote! { to_pubkey },
                "String" => quote! { to_string },
                _ => {
                    // For custom types, we need to create a closure that calls their deserialize_from
                    quote! { |data, pos| #type_name::deserialize_from(data, pos) }
                }
            }
        }
        _ => {
            panic!("Unsupported type for parser function generation");
        }
    }
}
