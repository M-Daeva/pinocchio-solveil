use {
    proc_macro::TokenStream,
    quote::quote,
    syn::{parse_macro_input, Data, DeriveInput, Fields, GenericArgument, PathArguments, Type},
};

extern crate proc_macro;

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
                return syn::Error::new_spanned(
                    &input.ident,
                    "ZCSerialize can only be derived for structs with named fields",
                )
                .to_compile_error()
                .into();
            }
            Fields::Unit => {
                quote! {
                    ByteWriter::new(data).complete()
                }
            }
        },
        _ => {
            return syn::Error::new_spanned(
                &input.ident,
                "ZCSerialize can only be derived for structs",
            )
            .to_compile_error()
            .into();
        }
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
        Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.last() {
                let type_name = segment.ident.to_string();

                // Check for Option first
                if type_name == "Option" {
                    if let PathArguments::AngleBracketed(args) = &segment.arguments {
                        if let Some(GenericArgument::Type(inner_type)) = args.args.first() {
                            return generate_option_write_call(field_name, inner_type);
                        }
                    }
                }

                // Handle primitive and known types
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
                        // For custom types that implement ZeroCopySerialize
                        quote! { .write_custom(&self.#field_name)? }
                    }
                }
            } else {
                quote! { .write_custom(&self.#field_name)? }
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
        Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.last() {
                let type_name = segment.ident.to_string();

                match type_name.as_str() {
                    "Pubkey" => {
                        quote! { .write_option(&self.#field_name, |v| v.as_ref())? }
                    }
                    "u8" => {
                        quote! { .write_option(&self.#field_name, |v| &[*v])? }
                    }
                    "u16" => {
                        quote! { .write_option(&self.#field_name, |v| &v.to_le_bytes())? }
                    }
                    "u32" => {
                        quote! { .write_option(&self.#field_name, |v| &v.to_le_bytes())? }
                    }
                    "u64" => {
                        quote! { .write_option(&self.#field_name, |v| &v.to_le_bytes())? }
                    }
                    "u128" => {
                        quote! { .write_option(&self.#field_name, |v| &v.to_le_bytes())? }
                    }
                    "bool" => {
                        quote! { .write_option(&self.#field_name, |v| &[if *v { 1u8 } else { 0u8 }])? }
                    }
                    "String" => {
                        quote! { .write_option(&self.#field_name, |v| v.as_bytes())? }
                    }
                    _ => {
                        // For custom types that implement ZeroCopySerialize + ZeroCopyDeserialize
                        quote! { .write_option_custom(&self.#field_name)? }
                    }
                }
            } else {
                quote! { .write_option_custom(&self.#field_name)? }
            }
        }
        _ => {
            // For complex Option types, use write_option_custom
            quote! { .write_option_custom(&self.#field_name)? }
        }
    }
}
