use {
    proc_macro::TokenStream,
    quote::quote,
    syn::{parse_macro_input, Data, DeriveInput, Fields, Type},
};

/// Derives a TryFrom<&[AccountInfo]> implementation for structs with AccountInfo fields
///
/// Usage:
/// ```rust
/// #[derive(AccountTryFrom)]
/// pub struct Accounts<'a> {
///     pub system_program: &'a AccountInfo,
///     pub token_program: &'a AccountInfo,
///     // ... other fields
/// }
/// ```
#[proc_macro_derive(AccountTryFrom)]
pub fn derive_account_try_from(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("AccountTryFrom only supports structs with named fields"),
        },
        _ => panic!("AccountTryFrom only supports structs"),
    };

    // Extract field names
    let field_names: Vec<_> = fields.iter().map(|f| f.ident.as_ref().unwrap()).collect();

    // Verify all fields are &AccountInfo references
    for field in fields {
        if let Type::Reference(type_ref) = &field.ty {
            if let Type::Path(type_path) = &*type_ref.elem {
                let last_segment = type_path.path.segments.last().unwrap();
                if last_segment.ident != "AccountInfo" {
                    panic!(
                        "All fields must be references to AccountInfo, found: {}",
                        quote!(#type_path)
                    );
                }
            } else {
                panic!(
                    "Expected AccountInfo reference, found: {}",
                    quote!(#type_ref)
                );
            }
        } else {
            panic!(
                "All fields must be references to AccountInfo, found: {}",
                quote!(#field.ty)
            );
        }
    }

    let field_count = field_names.len();

    // Generate the destructuring pattern
    let destructure_pattern = if field_count == 0 {
        quote! { [] }
    } else {
        quote! { [#(#field_names),*] }
    };

    // Generate the struct construction
    let struct_construction = quote! {
        Self {
            #(#field_names),*
        }
    };

    // Extract the lifetime parameter if present
    let lifetime = if let Some(lifetime_param) = generics.lifetimes().next() {
        let lifetime = &lifetime_param.lifetime;
        quote! { #lifetime }
    } else {
        quote! { '_ }
    };

    let expanded = quote! {
        impl #impl_generics TryFrom<& #lifetime [AccountInfo]> for #struct_name #ty_generics #where_clause {
            type Error = ProgramError;

            fn try_from(accounts: & #lifetime [AccountInfo]) -> core::result::Result<Self, Self::Error> {
                let #destructure_pattern = accounts
                else {
                    return Err(ProgramError::NotEnoughAccountKeys);
                };

                Ok(#struct_construction)
            }
        }
    };

    TokenStream::from(expanded)
}
