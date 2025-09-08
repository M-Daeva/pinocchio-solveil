use {
    proc_macro::TokenStream,
    quote::quote,
    syn::{parse_macro_input, Data, DeriveInput, Fields},
};

/// replaces \
/// #[derive(Default, Debug, PartialEq, Eq, Pod, Zeroable, Clone, Copy)] \
/// #[repr(C)] \
/// and implements DataLen
#[proc_macro_attribute]
pub fn p_serde(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let vis = &input.vis;
    let attrs = &input.attrs;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Handle the struct data properly
    let fields = match &input.data {
        Data::Struct(data_struct) => &data_struct.fields,
        _ => panic!("p_serde can only be used on structs"),
    };

    let struct_def = match fields {
        Fields::Named(fields) => {
            let named_fields = &fields.named;
            quote! {
                #[derive(Default, Debug, PartialEq, Eq, Pod, Zeroable, Clone, Copy)]
                #[repr(C)]
                #(#attrs)*
                #vis struct #name #generics {
                    #named_fields
                }
            }
        }
        Fields::Unnamed(fields) => {
            let unnamed_fields = &fields.unnamed;
            quote! {
                #[derive(Default, Debug, PartialEq, Eq, Pod, Zeroable, Clone, Copy)]
                #[repr(C)]
                #(#attrs)*
                #vis struct #name #generics(#unnamed_fields);
            }
        }
        Fields::Unit => {
            quote! {
                #[derive(Default, Debug, PartialEq, Eq, Pod, Zeroable, Clone, Copy)]
                #[repr(C)]
                #(#attrs)*
                #vis struct #name #generics;
            }
        }
    };

    let datalen_impl = quote! {
        impl #impl_generics DataLen for #name #ty_generics #where_clause {
            const LEN: usize = core::mem::size_of::<Self>();
        }
    };

    let expanded = quote! {
        #struct_def
        #datalen_impl
    };

    TokenStream::from(expanded)
}
