use {
    proc_macro::TokenStream,
    quote::quote,
    syn::{parse_macro_input, DeriveInput},
};

/// Procedural macro that generates InstructionSerialize implementation for testing
///
/// Usage:
/// ```
/// #[test_seri(Discriminator::WriteData)]
/// #[derive(Debug, PartialEq, Pod, Zeroable, Clone, Copy)]
/// #[repr(C)]
/// pub struct InstructionData {
///     // fields...
/// }
/// ```
#[proc_macro_attribute]
pub fn test_ser(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    // Parse the discriminator argument
    let discriminator = if args.is_empty() {
        panic!("test_ser macro requires a discriminator argument");
    } else {
        args.to_string()
    };

    // Parse the discriminator path to create a token stream
    let discriminator_tokens: proc_macro2::TokenStream =
        discriminator.parse().expect("Invalid discriminator path");

    // Generate the original struct
    let original_struct = quote! { #input };

    // Generate the InstructionSerialize implementation
    let serialize_impl = quote! {
        /// for tests
        #[cfg(feature = "dev")]
        impl base::traits::InstructionSerialize for #struct_name {
            fn serialize(&self) -> Result<Vec<u8>> {
                Ok([&[#discriminator_tokens as u8], bytemuck::bytes_of(self)].concat())
            }
        }
    };

    // Combine both the struct and the implementation
    let expanded = quote! {
        #original_struct

        #serialize_impl
    };

    TokenStream::from(expanded)
}
