#[proc_macro_derive(FromEnum)]
pub fn from_enum(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    proc_macro_helpers::panic_location::panic_location(); //panic_location function from https://github.com/kuqmua/proc_macro_helpers
    let ast: syn::DeriveInput = syn::parse_macro_input!(input as syn::DeriveInput);
    let ident = &ast.ident;
    let ident_with_serialize_deserialize_stringified = format!("{ident}WithSerializeDeserialize");
    let ident_with_serialize_deserialize_token_stream =
        ident_with_serialize_deserialize_stringified
            .parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| {
                panic!("FromEnum {ident} .parse::<proc_macro2::TokenStream>() failed")
            });
    let attribute_path = "from_enum::from_enum_paths";
    let option_attribute = ast.attrs.into_iter().find(|attr| {
        let possible_path = {
            let mut stringified_path = quote::ToTokens::to_token_stream(&attr.path).to_string();
            stringified_path.retain(|c| !c.is_whitespace());
            stringified_path
        };
        attribute_path == possible_path
    });
    let vec_enum_paths = if let Some(attribute) = option_attribute {
        let mut stringified_tokens =
            quote::ToTokens::to_token_stream(&attribute.tokens).to_string();
        stringified_tokens.retain(|c| !c.is_whitespace());
        match stringified_tokens.len() > 3 {
            true => {
                let mut chars = stringified_tokens.chars();
                match (chars.next(), chars.last()) {
                        (None, None) => panic!("FromEnum {ident} no first and last token attribute"),
                        (None, Some(_)) => panic!("FromEnum {ident} no first token attribute"),
                        (Some(_), None) => panic!("FromEnum {ident} no last token attribute"),
                        (Some(first), Some(last)) => match (first == '(', last == ')') {
                            (true, true) => {
                                match stringified_tokens.get(1..(stringified_tokens.len()-1)) {
                                    Some(inner_tokens_str) => {
                                        inner_tokens_str.split(',').map(|str|{str.to_string()}).collect::<Vec<std::string::String>>()
                                    },
                                    None => panic!("FromEnum {ident} cannot get inner_token"),
                                }
                            },
                            (true, false) => panic!("FromEnum {ident} last token attribute is not )"),
                            (false, true) => panic!("FromEnum {ident} first token attribute is not ("),
                            (false, false) => panic!("FromEnum {ident} first token attribute is not ( and last token attribute is not )"),
                        },
                    }
            }
            false => panic!("FromEnum {ident} stringified_tokens.len() > 3 == false"),
        }
    } else {
        panic!("{ident} FromEnum has no {attribute_path}");
    };
    let generated = vec_enum_paths.into_iter().map(|enum_path| {
        let enum_path_token_stream = enum_path
            .parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| {
                panic!("FromEnum {ident} {enum_path} .parse::<proc_macro2::TokenStream>() failed")
            });
        let variants = if let syn::Data::Enum(data_enum) = &ast.data {
            data_enum.variants.iter().map(|variant| {
                let variant_ident = &variant.ident;
                match &variant.fields {
                    syn::Fields::Named(fields_named) => {
                        let fields_generated = fields_named.named.iter().map(|field|{
                            field.ident.clone().unwrap_or_else(|| {
                                panic!("FromEnum {ident} {enum_path} field ident is None")
                            })
                        });
                        let fields_generated_cloned = fields_generated.clone();
                        quote::quote! {
                            #ident_with_serialize_deserialize_token_stream::#variant_ident { #(#fields_generated),* } => Self::#variant_ident { #(#fields_generated_cloned),* }
                        }
                    }
                    syn::Fields::Unnamed(_fields_unnamed) => {
                        // println!("{fields_unnamed:#?}");
                        // let fields_unnamed_iter = fields_unnamed.unnamed.iter();
                        // let fields_unnamed_iter_cloned = fields_unnamed_iter.clone();
                        // quote::quote! {
                        //     #ident_with_serialize_deserialize_token_stream::#variant_ident(#(#fields_unnamed_iter),*) => Self::#variant_ident(#(#fields_unnamed_iter_cloned),*)
                        // }
                        panic!(
                            "FromEnum {ident} logic for syn::Fields::Unnamed is not implemented yet"
                        )
                    }
                    syn::Fields::Unit => panic!(
                        "FromEnum {ident} works only with syn::Fields::Named and syn::Fields::Unnamed"
                    ),
                }
            })
        } else {
            panic!("FromEnum does work only on enums!");
        };
        let variant_gen = quote::quote! {
            impl std::convert::From<#ident_with_serialize_deserialize_token_stream>
                for #enum_path_token_stream
            {
                fn from(
                    val: #ident_with_serialize_deserialize_token_stream,
                ) -> Self {
                    match val {
                        #(#variants),*
                    }
                }
            }
        };
        // if enum_path == "" {
        //     println!("{variant_gen}");
        // }
        variant_gen
    });
    let gen = quote::quote! {
        #(#generated)*
    };
    // println!("{gen}");
    gen.into()
}

#[proc_macro_attribute]
pub fn from_enum_paths(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    item
}
