use {
    proc_macro::TokenStream,
    proc_macro2::TokenStream as TokenStream2,
    quote::{format_ident, quote},
    syn::{
        parse::Parse, parse::ParseStream, parse_macro_input, punctuated::Punctuated, Data,
        DeriveInput, Fields, Meta, Path, Token, Type, TypePath,
    },
};

#[proc_macro_derive(GenerateTS, attributes(ts_ignore, ts_name, enumfields))]
pub fn generate_ts(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let expanded = impl_generate_ts(&input);
    TokenStream::from(expanded)
}

fn impl_generate_ts(input: &DeriveInput) -> TokenStream2 {
    let struct_name = &input.ident;
    let ts_output = generate_typescript_code(input);

    // Generate a const that contains the TypeScript code
    let ts_const_name = format_ident!("TS_{}", struct_name.to_string().to_uppercase());

    quote! {
        const #ts_const_name: &str = #ts_output;

        impl #struct_name {
            pub fn typescript_interface() -> &'static str {
                #ts_const_name
            }
        }
    }
}

fn generate_typescript_code(input: &DeriveInput) -> String {
    let mut output = String::new();

    match &input.data {
        Data::Struct(data_struct) => {
            let struct_name = &input.ident;
            let is_enum_wrapper = has_enum_wrapper_derive(input);

            if is_enum_wrapper {
                output.push_str(&generate_enum_from_wrapper(input));
            } else {
                output.push_str(&generate_interface_and_codecs(
                    struct_name,
                    &data_struct.fields,
                ));
            }
        }
        _ => panic!("GenerateTS only supports structs"),
    }

    output
}

struct PathList(Punctuated<Path, Token![,]>);

impl Parse for PathList {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(PathList(input.parse_terminated(Path::parse, Token![,])?))
    }
}

fn has_enum_wrapper_derive(input: &DeriveInput) -> bool {
    input.attrs.iter().any(|attr| {
        if let Meta::List(meta_list) = &attr.meta {
            return attr.path().is_ident("derive") && {
                let tokens = &meta_list.tokens;
                let parsed: Result<PathList, _> = syn::parse2(tokens.clone());
                if let Ok(PathList(paths)) = parsed {
                    paths.iter().any(|path| path.is_ident("EnumWrapper"))
                } else {
                    false
                }
            };
        }
        false
    })
}

fn generate_enum_from_wrapper(input: &DeriveInput) -> String {
    let enum_name = &input.ident;

    // Find enumfields attribute
    let enum_fields = input
        .attrs
        .iter()
        .find_map(|attr| {
            if attr.path().is_ident("enumfields") {
                // Parse the enumfields content
                if let Meta::List(meta_list) = &attr.meta {
                    let tokens = &meta_list.tokens;
                    let parsed: Result<PathList, _> = syn::parse2(tokens.clone());
                    if let Ok(PathList(paths)) = parsed {
                        let fields: Vec<String> = paths
                            .iter()
                            .filter_map(|path| {
                                path.get_ident().map(|ident| {
                                    // Convert snake_case to PascalCase
                                    let s = ident.to_string();
                                    s.split('_')
                                        .map(|word| {
                                            let mut chars = word.chars();
                                            match chars.next() {
                                                None => String::new(),
                                                Some(first) => {
                                                    first.to_uppercase().collect::<String>()
                                                        + chars.as_str()
                                                }
                                            }
                                        })
                                        .collect::<Vec<_>>()
                                        .join("")
                                })
                            })
                            .collect();
                        Some(fields)
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        })
        .unwrap_or_default();

    let mut result = format!("export enum {} {{\n", enum_name);
    for field in enum_fields {
        result.push_str(&format!("  {},\n", field));
    }
    result.push_str("}\n\n");
    result
}

fn generate_interface_and_codecs(struct_name: &syn::Ident, fields: &Fields) -> String {
    let mut output = String::new();

    // Generate interface
    let interface_name = format!("I{}", struct_name);
    output.push_str(&format!("export interface {} {{\n", interface_name));

    if let Fields::Named(fields_named) = fields {
        for field in &fields_named.named {
            let field_name = field.ident.as_ref().unwrap();
            let ts_type = rust_type_to_ts_type(&field.ty);
            let ts_field_name = snake_to_camel(&field_name.to_string());
            output.push_str(&format!("  {}: {};\n", ts_field_name, ts_type));
        }
    }
    output.push_str("}\n\n");

    // Generate encoder function
    let enc_function_name = format!("enc{}", struct_name);
    output.push_str(&format!(
        "export function {}(x?: {}): {} {{\n",
        enc_function_name, interface_name, struct_name
    ));
    output.push_str("  return {\n");

    if let Fields::Named(fields_named) = fields {
        for field in &fields_named.named {
            let field_name = field.ident.as_ref().unwrap();
            let ts_field_name = snake_to_camel(&field_name.to_string());
            let encoder_expr = generate_field_encoder(&field.ty, &ts_field_name);
            output.push_str(&format!("    {}: {},\n", ts_field_name, encoder_expr));
        }
    }
    output.push_str("  };\n}\n\n");

    // Generate decoder function
    let dec_function_name = format!("dec{}", struct_name);
    output.push_str(&format!(
        "export function {}(x?: {}): {} {{\n",
        dec_function_name, struct_name, interface_name
    ));
    output.push_str("  return {\n");

    if let Fields::Named(fields_named) = fields {
        for field in &fields_named.named {
            let field_name = field.ident.as_ref().unwrap();
            let ts_field_name = snake_to_camel(&field_name.to_string());
            let decoder_expr = generate_field_decoder(&field.ty, &ts_field_name);
            output.push_str(&format!("    {}: {},\n", ts_field_name, decoder_expr));
        }
    }
    output.push_str("  };\n}\n\n");

    output
}

fn rust_type_to_ts_type(ty: &Type) -> String {
    match ty {
        Type::Path(TypePath { path, .. }) => {
            let type_name = path.segments.last().unwrap().ident.to_string();
            match type_name.as_str() {
                "Pubkey" => "Address".to_string(),
                "u8" => "number".to_string(),
                "Uint16" | "Uint32" => "number".to_string(),
                "Uint64" | "Uint128" => "bigint".to_string(),
                "String16" | "String32" | "String64" | "String4096" => "string".to_string(),
                "BitField" => "boolean".to_string(),
                _ => {
                    // Check if it's a custom struct that should have an I prefix
                    if is_custom_struct(&type_name) {
                        format!("I{}", type_name)
                    } else {
                        type_name
                    }
                }
            }
        }
        _ => "unknown".to_string(),
    }
}

fn generate_field_encoder(ty: &Type, field_name: &str) -> String {
    if let Type::Path(TypePath { path, .. }) = ty {
        let type_name = path.segments.last().unwrap().ident.to_string();
        match type_name.as_str() {
            "Pubkey" => format!("x?.{} || DEFAULT_ADDRESS", field_name),
            "u8" => format!("x?.{} || 0", field_name),
            "BitField" => format!(
                "new TBitField({{ flag: x?.{} || false }}).getRaw()",
                field_name
            ),
            "Uint16" => format!("new TUint16(x?.{}).getRaw()", field_name),
            "Uint32" => format!("new TUint32(x?.{}).getRaw()", field_name),
            "Uint64" => format!("new TUint64(x?.{}).getRaw()", field_name),
            "Uint128" => format!("new TUint128(x?.{}).getRaw()", field_name),
            "String16" | "String32" | "String64" | "String4096" => {
                format!("new T{}(x?.{}).getRaw()", type_name, field_name)
            }
            _ => {
                if is_custom_struct(&type_name) {
                    format!("enc{}(x?.{})", type_name, field_name)
                } else {
                    format!("x?.{}", field_name)
                }
            }
        }
    } else {
        format!("x?.{}", field_name)
    }
}

fn generate_field_decoder(ty: &Type, field_name: &str) -> String {
    if let Type::Path(TypePath { path, .. }) = ty {
        let type_name = path.segments.last().unwrap().ident.to_string();
        match type_name.as_str() {
            "Pubkey" => format!("x?.{} || DEFAULT_ADDRESS", field_name),
            "u8" => format!("x?.{} || 0", field_name),
            "BitField" => format!("new TBitField({{ value: x?.{} || 0 }}).get()", field_name),
            "Uint16" => format!("new TUint16(x?.{}).get()", field_name),
            "Uint32" => format!("new TUint32(x?.{}).get()", field_name),
            "Uint64" => format!("new TUint64(x?.{}).get()", field_name),
            "Uint128" => format!("new TUint128(x?.{}).get()", field_name),
            "String16" | "String32" | "String64" | "String4096" => {
                format!("new T{}(x?.{}).get()", type_name, field_name)
            }
            _ => {
                if is_custom_struct(&type_name) {
                    format!("dec{}(x?.{})", type_name, field_name)
                } else {
                    format!("x?.{}", field_name)
                }
            }
        }
    } else {
        format!("x?.{}", field_name)
    }
}

fn is_custom_struct(type_name: &str) -> bool {
    !matches!(
        type_name,
        "Pubkey"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "u128"
            | "Uint16"
            | "Uint32"
            | "Uint64"
            | "Uint128"
            | "BitField"
            | "String16"
            | "String32"
            | "String64"
            | "String4096"
            | "Address"
            | "number"
            | "bigint"
            | "string"
            | "boolean"
    )
}

fn snake_to_camel(snake_str: &str) -> String {
    let mut camel = String::new();
    let mut capitalize_next = false;

    for (_i, ch) in snake_str.chars().enumerate() {
        if ch == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            camel.push(ch.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            camel.push(ch);
        }
    }

    camel
}
