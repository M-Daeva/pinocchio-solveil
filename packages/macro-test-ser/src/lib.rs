use {
    proc_macro::TokenStream,
    proc_macro2::TokenStream as TokenStream2,
    quote::quote,
    syn::{parse_macro_input, Data, DeriveInput, Fields, Type, TypePath},
};

/// Derive macro for automatically implementing InstructionSerialize with discriminator
/// Usage:
/// ```
/// #[derive(Default)]
/// #[test_serialize(DISCRIMINATOR::INIT)]
/// pub struct InstructionData { ... }
/// ```
#[proc_macro_attribute]
pub fn test_serialize(args: TokenStream, input: TokenStream) -> TokenStream {
    let discriminator = args.to_string();
    let input_parsed = parse_macro_input!(input as DeriveInput);

    let name = &input_parsed.ident;
    let generics = &input_parsed.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let discriminator_path: syn::Path =
        syn::parse_str(&discriminator).expect("Invalid discriminator path");

    let serialize_body = match &input_parsed.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields) => {
                let field_writes = fields.named.iter().map(|field| {
                    let field_name = &field.ident;
                    let field_type = &field.ty;

                    generate_test_write_call(field_name.as_ref().unwrap(), field_type)
                });

                quote! {
                    use base::converters::{u32_as_bytes, ByteWriter, ByteWriterVecExt};

                    let mut buffer = vec![];
                    let position = ByteWriter::from_vec(&mut buffer)
                        .write_u8(#discriminator_path)?
                        #(#field_writes)*
                        .position();
                    buffer.truncate(position);

                    Ok(buffer)
                }
            }
            Fields::Unit => {
                quote! {
                    use base::converters::{ByteWriter, ByteWriterVecExt};

                    let mut buffer = vec![];
                    let position = ByteWriter::from_vec(&mut buffer)
                        .write_u8(#discriminator_path)?
                        .position();
                    buffer.truncate(position);

                    Ok(buffer)
                }
            }
            _ => panic!("test_serialize can only be used with structs with named fields"),
        },
        _ => panic!("test_serialize can only be used with structs"),
    };

    let serialize_impl = quote! {
        #[cfg(feature = "dev")]
        impl #impl_generics base::types::InstructionSerialize for #name #ty_generics #where_clause {
            fn serialize(&self) -> Result<Vec<u8>> {
                #serialize_body
            }
        }
    };

    let expanded = quote! {
        #input_parsed
        #serialize_impl
    };

    TokenStream::from(expanded)
}

fn generate_test_write_call(field_name: &syn::Ident, field_type: &Type) -> TokenStream2 {
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
                                    return generate_test_option_write_call(field_name, inner_type);
                                }
                            }
                        }
                    }

                    // For custom types - use write_option_custom as shown in your example
                    quote! { .write_option_custom(&self.#field_name)? }
                }
            }
        }
        _ => {
            // For any other complex types
            quote! { .write_option_custom(&self.#field_name)? }
        }
    }
}

fn generate_test_option_write_call(field_name: &syn::Ident, inner_type: &Type) -> TokenStream2 {
    match inner_type {
        Type::Path(TypePath { path, .. }) => {
            let type_name = path.segments.last().unwrap().ident.to_string();

            match type_name.as_str() {
                "u32" => {
                    // Special case for u32 as shown in your example
                    quote! { .write_option(&self.#field_name, u32_as_bytes)? }
                }
                "u8" => {
                    quote! { .write_option(&self.#field_name, |v| [*v])? }
                }
                "u16" => {
                    quote! { .write_option(&self.#field_name, |v| v.to_le_bytes())? }
                }
                "u64" => {
                    quote! { .write_option(&self.#field_name, |v| v.to_le_bytes())? }
                }
                "u128" => {
                    quote! { .write_option(&self.#field_name, |v| v.to_le_bytes())? }
                }
                "bool" => {
                    quote! { .write_option(&self.#field_name, |v| [*v as u8])? }
                }
                "Pubkey" => {
                    quote! { .write_option(&self.#field_name, |v| v.as_ref())? }
                }
                "String" => {
                    quote! { .write_option(&self.#field_name, |v| v.as_bytes())? }
                }
                _ => {
                    // For custom types like AssetItem, Range - use write_option_custom
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
