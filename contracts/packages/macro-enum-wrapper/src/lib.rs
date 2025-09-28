use {
    proc_macro::TokenStream,
    quote::quote,
    syn::{parse_macro_input, Data, DeriveInput, Ident, Variant},
};

/// Derives a wrapper struct for an enum that stores the enum as its underlying representation
///
/// The generated wrapper struct will have:
/// - The same name as the enum with "Enum" suffix
/// - A From<EnumType> implementation
/// - get_raw() and set_raw() methods for direct access to the underlying value
/// - get() and set() methods for converting to/from the original enum
#[proc_macro_derive(EnumWrapper)]
pub fn derive_enum_wrapper(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let enum_name = &input.ident;
    let wrapper_name = Ident::new(&format!("{}Enum", enum_name), enum_name.span());

    // Determine the underlying type from #[repr(...)]
    let underlying_type = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("repr"))
        .and_then(|attr| attr.parse_args::<Ident>().ok())
        .unwrap_or_else(|| Ident::new("u8", enum_name.span()));

    // Extract enum variants
    let variants = match &input.data {
        Data::Enum(data_enum) => &data_enum.variants,
        _ => panic!("EnumWrapper can only be used on enums"),
    };

    // Generate match arms for the get() method
    let match_arms = variants.iter().enumerate().map(|(index, variant)| {
        let variant_name = &variant.ident;
        let discriminant = get_variant_discriminant(variant, index);

        quote! {
            #discriminant => #enum_name::#variant_name,
        }
    });

    // Check if enum has #[derive(...)] attributes to copy relevant ones
    let derive_attrs = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("derive"))
        .map(|attr| {
            // Check if CodamaType is in the derive list
            let attr_str = quote!(#attr).to_string();
            if attr_str.contains("CodamaType") {
                quote! { #[derive(CodamaType)] }
            } else {
                quote! {}
            }
        })
        .unwrap_or_else(|| quote! {});

    let expanded = quote! {
        #derive_attrs
        #[p_serde]
        pub struct #wrapper_name(#underlying_type);

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

fn get_variant_discriminant(variant: &Variant, index: usize) -> proc_macro2::TokenStream {
    // Check if the variant has an explicit discriminant
    if let Some((_, expr)) = &variant.discriminant {
        quote! { #expr }
    } else {
        // Use the index as the discriminant value
        let index_lit = proc_macro2::Literal::usize_unsuffixed(index);
        quote! { #index_lit }
    }
}
