use {
    proc_macro::TokenStream,
    quote::quote,
    syn::{parse_macro_input, Data, DeriveInput, ExprPath, Field, Fields},
};

#[proc_macro_derive(OptionFlag, attributes(optional))]
pub fn derive_option_flag(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Extract fields with #[optional(flags)] attribute
    let optional_fields = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields) => fields
                .named
                .iter()
                .filter_map(|field| {
                    if has_optional_flags_attribute(field) {
                        Some(field)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>(),
            _ => panic!("OptionFlag can only be derived for structs with named fields"),
        },
        _ => panic!("OptionFlag can only be derived for structs"),
    };

    // Generate constants and methods
    let mut constants = Vec::new();
    let mut methods = Vec::new();

    for (index, field) in optional_fields.iter().enumerate() {
        let field_name = field.ident.as_ref().unwrap();
        let const_name = field_name.to_string().to_uppercase();
        let const_ident = syn::Ident::new(&const_name, field_name.span());

        let getter_name = syn::Ident::new(&format!("get_{}_flag", field_name), field_name.span());
        let setter_name = syn::Ident::new(&format!("set_{}_flag", field_name), field_name.span());

        // Generate constant
        let index_u8 = index as u8;
        constants.push(quote! {
            const #const_ident: u8 = #index_u8;
        });

        // Generate getter method
        methods.push(quote! {
            #[inline]
            pub fn #getter_name(&self) -> bool {
                self.flags.get_flag(Self::#const_ident)
            }
        });

        // Generate setter method
        methods.push(quote! {
            #[inline]
            pub fn #setter_name(&mut self, x: bool) {
                self.flags.set_flag(Self::#const_ident, x);
            }
        });
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

fn has_optional_flags_attribute(field: &Field) -> bool {
    for attr in &field.attrs {
        if attr.path().is_ident("optional") {
            if let Ok(args) = attr.parse_args::<ExprPath>() {
                if args.path.is_ident("flags") {
                    return true;
                }
            }
        }
    }
    false
}
