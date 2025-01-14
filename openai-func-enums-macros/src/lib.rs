use proc_macro::{TokenStream, TokenTree};
// use proc_macro2::Group;
use quote::{format_ident, quote, ToTokens};
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Expr, Ident, Lit, Meta};
use tiktoken_rs::cl100k_base;

/// The `arg_description` attribute is a procedural macro used to provide additional description for an enum.
///
/// This attribute does not modify the code it annotates but instead attaches metadata in the form of a description.
/// This can be helpful for better code readability and understanding the purpose of different enums.
///
/// # Usage
///
/// ```rust
/// #[arg_description(description = "This is a sample enum.", tokens = 5)]
/// #[derive(EnumDescriptor)]
/// pub enum SampleEnum {
///     Variant1,
///     Variant2,
/// }
/// ```
///
/// Note: The actual usage of the description and tokens provided through this attribute happens
/// in the `EnumDescriptor` derive macro and is retrieved in the `enum_descriptor_derive` function.
///
/// The `arg_description` attribute takes one argument, `description`, which is a string literal.
#[proc_macro_attribute]
pub fn arg_description(_args: TokenStream, input: TokenStream) -> TokenStream {
    input
}

/// A derive procedural macro for the `EnumDescriptor` trait.
///
/// The `EnumDescriptor` trait should have a function `name_with_token_count`
/// that returns a tuple with the name of the enum type as a string and the
/// token count for the name as an `usize`.
///
/// This procedural macro generates an implementation of `EnumDescriptor` for
/// the type on which it's applied. The `name_with_token_count` function, in the
/// generated implementation, returns the name of the type and its token count.
///
/// # Usage
///
/// Use the `#[derive(EnumDescriptor)]` attribute on an enum to derive the
/// `EnumDescriptor` trait for it.
///
/// ```
/// #[derive(EnumDescriptor)]
/// enum MyEnum {
///     Variant1,
///     Variant2,
/// }
/// ```
///
/// This will generate:
///
/// ```
/// impl EnumDescriptor for MyEnum {
///     fn name_with_token_count() -> (String, usize) {
///         (String::from("MyEnum"), /* token count of "MyEnum" */)
///     }
/// }
/// ```
///
/// The actual token count is computed during compile time using the
/// `calculate_token_count` function.
#[proc_macro_derive(EnumDescriptor, attributes(arg_description))]
pub fn enum_descriptor_derive(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, attrs, .. } = parse_macro_input!(input as DeriveInput);

    let name_str = format!("{}", ident);
    let name_token_count = calculate_token_count(&name_str);

    let mut description = String::new();
    let mut desc_tokens = 0_usize;

    for attr in &attrs {
        if attr.path().is_ident("arg_description") {
            let _result = attr.parse_nested_meta(|meta| {
                let content = meta.input;

                if content.is_empty() {
                    // return Err(meta.error("expected `description` and `tokens`"));
                    return Err(meta.error("unrecognized my_attribute"));
                }

                // while !content.is_empty() {
                if meta.path.is_ident("description") {
                    let value = meta.value()?;
                    if let Ok(Lit::Str(value)) = value.parse() {
                        description = value.value();
                    }
                } else if meta.path.is_ident("tokens") {
                    let value = meta.value()?;
                    if let Ok(Lit::Int(value)) = value.parse() {
                        desc_tokens = value.base10_parse::<usize>()?;
                        return Ok(());
                    }
                }

                Ok(())
                // }
            });

            if _result.is_err() {
                println!("Error parsing attribute:   {:#?}", _result);
            }
        }
    }

    let expanded = quote! {
        impl EnumDescriptor for #ident {
            fn name_with_token_count() -> (String, usize) {
                (String::from(#name_str), #name_token_count)
            }
            fn arg_description_with_token_count() -> (String, usize) {
                (String::from(#description), #desc_tokens)
            }
        }
    };

    TokenStream::from(expanded)
}

/// A derive procedural macro for the `VariantDescriptors` trait.
///
/// This macro generates an implementation of the `VariantDescriptors` trait for
/// an enum. The trait provides two methods:
///
/// 1. `variant_names_with_token_counts`: Returns a `Vec` containing tuples,
/// each with a string representation of a variant's name and its token count.
///
/// 2. `variant_name_with_token_count`: Takes an enum variant as input and
/// returns a tuple with the variant's name as a string and its token count.
///
/// Note: This macro will panic if it is used on anything other than an enum.
///
/// # Usage
///
/// ```
/// #[derive(VariantDescriptors)]
/// enum MyEnum {
///     Variant1,
///     Variant2,
/// }
/// ```
///
/// This will generate the following:
///
/// ```
/// impl VariantDescriptors for MyEnum {
///     fn variant_names_with_token_counts() -> Vec<(String, usize)> {
///         vec![
///             (String::from("Variant1"), /* token count of "Variant1" */),
///             (String::from("Variant2"), /* token count of "Variant2" */),
///         ]
///     }
///
///     fn variant_name_with_token_count(&self) -> (String, usize) {
///         match self {
///             Self::Variant1 => (String::from("Variant1"), /* token count of "Variant1" */),
///             Self::Variant2 => (String::from("Variant2"), /* token count of "Variant2" */),
///         }
///     }
/// }
/// ```
///
/// The actual token count is computed during compile time using the
/// `calculate_token_count` function.
#[proc_macro_derive(VariantDescriptors)]
pub fn variant_descriptors_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let enum_name = &ast.ident;

    let variants = if let syn::Data::Enum(ref e) = ast.data {
        e.variants
            .iter()
            .map(|v| {
                let variant_name = &v.ident;
                let token_count = calculate_token_count(&variant_name.to_string());

                (variant_name, token_count)
            })
            .collect::<Vec<_>>()
    } else {
        panic!("VariantDescriptors can only be used with enums");
    };

    let variant_names_with_token_counts: Vec<_> = variants
        .iter()
        .map(|(variant_name, token_count)| {
            quote! { (stringify!(#variant_name).to_string(), #token_count) }
        })
        .collect();

    let variant_name_with_token_count: Vec<_> = variants
        .iter()
        .map(|(variant_name, token_count)| {
            quote! { Self::#variant_name => (stringify!(#variant_name).to_string(), #token_count) }
        })
        .collect();

    let expanded = quote! {
        impl VariantDescriptors for #enum_name {
            fn variant_names_with_token_counts() -> Vec<(String, usize)> {
                vec![
                    #(#variant_names_with_token_counts),*
                ]
            }

            fn variant_name_with_token_count(&self) -> (String, usize) {
                match self {
                    #(#variant_name_with_token_count,)*
                }
            }
        }
    };

    TokenStream::from(expanded)
}

/// A procedural macro to generate information about an enum.
///
/// This macro generates code that uses the `EnumDescriptor` and `VariantDescriptors`
/// traits to extract information about an enum, including its name, variant names,
/// and their corresponding token counts. Additionally, it uses the `FunctionArgument` trait
/// to fetch the argument description. All this information is serialized into JSON.
///
/// The macro returns a tuple containing the JSON and the total token count.
///
/// # Usage
///
/// The generated code will look like this:
///
/// ```rust
/// {
///     use serde_json::Value;
///     let mut total_tokens = 0;
///
///     let (arg_desc, arg_count) = <MyEnum as ::openai_func_enums::FunctionArgument>::argument_description_with_token_count();
///     total_tokens += arg_count;
///
///     // When this is consumed by the function that creates the overall function,
///     // we are going to be requiring all the arguments, which means we will repeat
///     // their names in the "required" part of openai's function schema. So we will
///     // count the tokens associated with this enum name twice here.
///     let enum_name = <MyEnum as EnumDescriptor>::name_with_token_count();
///     total_tokens += enum_name.1;
///     total_tokens += enum_name.1;
///
///     let enum_variants = <MyEnum as VariantDescriptors>::variant_names_with_token_counts();
///     total_tokens += enum_variants.iter().map(|(_, token_count)| *token_count).sum::<usize>();
///
///     let json_enum = serde_json::json!({
///         enum_name.0: {
///             "type": "string",
///             "enum": enum_variants.iter().map(|(name, _)| name.clone()).collect::<Vec<_>>(),
///             "description": arg_desc,
///         }
///     });
///
///     total_tokens += 11;
///
///     (json_enum, total_tokens)
/// }
/// ```
///
/// Note: It is assumed that the enum implements the `EnumDescriptor` and `VariantDescriptors` traits.
/// The actual token count is computed during compile time using these traits' methods.
#[proc_macro]
pub fn generate_enum_info(input: TokenStream) -> TokenStream {
    let enum_ident = parse_macro_input!(input as Ident);

    let output = quote! {
        {
            use serde_json::Value;
            let mut total_tokens = 0;

            // let (arg_desc, arg_count) = <#enum_ident as ::openai_func_enums::FunctionArgument>::argument_description_with_token_count();
            let (arg_desc, arg_count) = <#enum_ident as EnumDescriptor>::arg_description_with_token_count();
            total_tokens += arg_count;

            let enum_name = <#enum_ident as EnumDescriptor>::name_with_token_count();
            total_tokens += enum_name.1;
            total_tokens += enum_name.1;

            let enum_variants = <#enum_ident as VariantDescriptors>::variant_names_with_token_counts();
            total_tokens += enum_variants.iter().map(|(_, token_count)| *token_count).sum::<usize>();

            let json_enum = serde_json::json!({
                enum_name.0: {
                    "type": "string",
                    "enum": enum_variants.iter().map(|(name, _)| name.clone()).collect::<Vec<_>>(),
                    "description": arg_desc,
                }
            });

            total_tokens += 11;

            (json_enum, total_tokens)
        }
    };

    output.into()
}

#[proc_macro]
// pub fn generate_value_arg_info(input: TokenStream, arg_name: TokenStream) -> TokenStream {
pub fn generate_value_arg_info(input: TokenStream) -> TokenStream {
    // println!("Got here 100");
    // let value_ident = parse_macro_input!(input as Punctuated<LitStr, Token![,]>);
    // for arg in value_ident.into() {
    //     if arg.is_ident() {
    //         println!("this is an ident: {:#?}", arg);
    //     }
    // }

    // for arg in input.into() {
    //     let test = arg;
    // }

    let mut type_and_name_values = Vec::new();

    let tokens = input.into_iter().collect::<Vec<TokenTree>>();
    // println!("Got here 200");
    for token in tokens {
        // match &token {
        //     TokenTree::Ident(ident) => {
        //         type_and_name_values.push(ident.to_string());
        //     }
        //     _ => {}
        // }

        if let TokenTree::Ident(ident) = &token {
            type_and_name_values.push(ident.to_string());
        }
    }
    // let arg_name_ident = parse_macro_input!(arg_name as Ident);
    // println!("value_ident: {:#?}", value_ident);
    // let value_ident_tokens = calculate_token_count(value_ident.)
    // let name = value_ident.to_string();
    // let name = String::from("test");
    // let name_tokens = calculate_token_count(name.as_str());

    if type_and_name_values.len() == 2 {
        let name = type_and_name_values[1].clone();
        let name_tokens = calculate_token_count(name.as_str());
        let type_name = type_and_name_values[0].clone();
        let type_name_tokens = calculate_token_count(type_name.as_str());
        let output = quote! {
            {
                use serde_json::Value;
                let mut total_tokens = 0;
                total_tokens += #name_tokens;
                total_tokens += #type_name_tokens;
                //
                // let (arg_desc, arg_count) = <#enum_ident as EnumDescriptor>::arg_description_with_token_count();
                // total_tokens += arg_count;
                //
                // let enum_name = <#enum_ident as EnumDescriptor>::name_with_token_count();
                // total_tokens += enum_name.1;
                // total_tokens += enum_name.1;
                //
                // let enum_variants = <#enum_ident as VariantDescriptors>::variant_names_with_token_counts();
                // total_tokens += enum_variants.iter().map(|(_, token_count)| *token_count).sum::<usize>();
                //
                let json_enum = serde_json::json!({
                    #name: {
                        "type": #type_name,
                    }
                });

                total_tokens += 11;

                (json_enum, total_tokens)
            }
        };
        return output.into();
    }

    // output.into()
    let gen = quote! {};

    gen.into()
}
/// This procedural macro attribute is used to specify a description for an enum variant.
///
/// The `func_description` attribute does not modify the input it is given.
/// It's only used to attach metadata (i.e., a description) to enum variants.
///
/// # Usage
///
/// ```rust
/// enum MyEnum {
///     #[func_description(description="This function does a thing.")]
///     DoAThing,
///     #[func_description(description="This function does another thing.")]
///     DoAnotherThing,
/// }
/// ```
///
/// Note: The actual usage of the description provided through this attribute happens
/// in the `FunctionCallResponse` derive macro and is retrieved in the `impl_function_call_response` function.
#[proc_macro_attribute]
pub fn func_description(_args: TokenStream, input: TokenStream) -> TokenStream {
    input
}

/// This procedural macro derives the `FunctionCallResponse` trait for an enum.
///
/// The derive macro expects an enum and it generates a new struct for each variant of the enum.
/// The generated struct is named by appending "Response" to the variant's name. Each struct has the same fields as the variant.
/// Also, a `name`, `to_function_call` and `get_function_json` method is implemented for each struct.
///
/// In the `get_function_json` method, any description provided through the `func_description` attribute is used.
///
/// # Usage
///
/// ```rust
/// #[derive(FunctionCallResponse)]
/// #[func_description]
/// enum MyEnum {
///     Variant1,
///     Variant2,
/// }
/// ```
///
/// Note: This macro can only be applied to enums and it requires the `func_description` attribute to be applied to the enum.
#[proc_macro_derive(FunctionCallResponse, attributes(func_description))]
pub fn derive_function_call_response(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();

    let gen = impl_function_call_response(&ast);

    gen.into()
}

/// This function generates a `FunctionCallResponse` implementation for each variant of an enum.
///
/// For each enum variant, it creates a new struct with the same fields as the variant and also
/// generates `name`, `to_function_call`, and `get_function_json` methods for the struct.
///
/// In the `get_function_json` method, it utilizes the description provided through the `func_description` attribute.
///
/// This function is used by the `FunctionCallResponse` derive macro.
fn impl_function_call_response(ast: &DeriveInput) -> proc_macro2::TokenStream {
    match &ast.data {
        Data::Enum(enum_data) => {
            let mut generated_structs = Vec::new();
            let mut json_generator_functions = Vec::new();

            for variant in &enum_data.variants {
                let variant_name = &variant.ident;
                let struct_name = format_ident!("{}Response", variant_name);

                let mut description = String::new();
                let mut desc_tokens = 0_usize;

                for attr in &variant.attrs {
                    if attr.path().is_ident("func_description") {
                        let attribute_parsed = attr.parse_nested_meta(|meta| {
                            let content = meta.input;

                            if content.is_empty() {
                                // return Err(meta.error("expected `description` and `tokens`"));
                                return Err(meta.error("unrecognized my_attribute"));
                            }

                            // while !content.is_empty() {
                            if meta.path.is_ident("description") {
                                let value = meta.value()?;
                                if let Ok(Lit::Str(value)) = value.parse() {
                                    description = value.value();
                                    desc_tokens = calculate_token_count(description.as_str());
                                }
                            }

                            Ok(())
                            // }
                        });
                        match attribute_parsed {
                            Ok(_attribute_parsed) => {}
                            Err(e) => {
                                println!("Error parsing attribute:   {:#?}", e);
                            }
                        }
                    }
                }

                let fields: Vec<_> = variant
                    .fields
                    .iter()
                    .map(|f| {
                        let field_name =
                            format_ident!("{}", to_snake_case(&f.ty.to_token_stream().to_string()));
                        let field_type = &f.ty;
                        quote! {
                            pub #field_name: #field_type,
                        }
                    })
                    .collect();

                let field_info: Vec<_> = variant
                    .fields
                    .iter()
                    .map(|f| {
                        let field_type = &f.ty;
                        quote! {
                            generate_enum_info!(#field_type)
                        }
                    })
                    .collect();

                json_generator_functions.push(quote! {
                    impl #struct_name {
                        pub fn name() -> String {
                            stringify!(#struct_name).to_string()
                        }

                        pub fn to_function_call() -> ChatCompletionFunctionCall {
                            let function_call_json = json!({
                                "name": stringify!(#struct_name)
                            });

                            ChatCompletionFunctionCall::Object(function_call_json)
                        }

                        pub fn get_function_json() -> (Value, usize) {
                            let mut parameters = serde_json::Map::new();
                            let mut total_tokens = 0;
                            for (arg_json, arg_tokens) in vec![#(#field_info),*] {
                                total_tokens += arg_tokens;
                                parameters.insert(
                                    arg_json.as_object().unwrap().keys().next().unwrap().clone(),
                                    arg_json
                                        .as_object()
                                        .unwrap()
                                        .values()
                                        .next()
                                        .unwrap()
                                        .clone(),
                                );
                            }

                            let function_json = json!({
                                "name": stringify!(#struct_name),
                                "description": #description,
                                "parameters": {
                                    "type": "object",
                                    "properties": parameters,
                                    "required": parameters.keys().collect::<Vec<_>>()
                                }
                            });

                            total_tokens += 12;
                            total_tokens += #desc_tokens;

                            (function_json, total_tokens)
                        }
                    }
                });

                generated_structs.push(quote! {
                    #[derive(serde::Deserialize, Debug)]
                    #[serde(rename_all = "PascalCase")]
                    pub struct #struct_name {
                        #(#fields)*
                    }
                });
            }

            let gen = quote! {
                #(#generated_structs)*

                #(#json_generator_functions)*

            };

            gen
        }
        _ => panic!("FunctionCallResponse can only be derived for enums"),
    }
}

/// The `SubcommandGPT` procedural macro is used to derive a structure
/// which encapsulates various chat completion commands.
///
/// This macro should be applied to an enum. It generates various supporting
/// structures and methods, including structures representing the command arguments,
/// methods for converting between the argument structures and the original enum,
/// JSON conversion methods, and an implementation of the original enum that provides
/// methods for executing the commands and dealing with the responses.
///
/// Each variant of the original enum will be converted into a corresponding structure,
/// and each field in the variant will become a field in the generated structure.
/// The generated structures will derive `serde::Deserialize` and `Debug` automatically.
///
/// This macro also generates methods for calculating the token count of a string and
/// for executing commands based on function calls received from the chat API.
///
/// The types of fields in the enum variants determine how the corresponding fields in the
/// generated structures are treated. For example, fields of type `String` or `&str` are
/// converted to JSON value arguments with type `"string"`, while fields of type `u8`, `u16`,
/// `u32`, `u64`, `usize`, `i8`, `i16`, `i32`, `i64`, `isize`, `f32` or `f64` are converted
/// to JSON value arguments with type `"integer"` or `"number"` respectively.
/// For fields with a tuple type, currently this macro simply prints that the field is of a tuple type.
/// For fields with an array type, they are converted to JSON value arguments with type `"array"`.
///
/// When running the chat command, a custom system message can be optionally provided.
/// If provided, this message will be used as the system message in the chat request.
/// If not provided, a default system message will be used.
///
/// If the total token count of the request exceeds a specified limit, an error will be returned.
///
/// The `derive_subcommand_gpt` function consumes a `TokenStream` representing the enum
/// to which the macro is applied and produces a `TokenStream` representing the generated code.
///
/// # Panics
/// This macro will panic (only at compile time) if it is applied to a non-enum item.
#[proc_macro_derive(SubcommandGPT)]
pub fn derive_subcommand_gpt(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let data = match input.data {
        Data::Enum(data) => data,
        _ => panic!("SubcommandGPT can only be implemented for enums"),
    };

    let mut generated_structs = Vec::new();
    let mut json_generator_functions = Vec::new();
    let mut generated_clap_gpt_enum = Vec::new();
    let mut generated_struct_names = Vec::new();

    for variant in data.variants.iter() {
        let variant_name = &variant.ident;
        let struct_name = format_ident!("{}Response", variant_name);
        generated_struct_names.push(struct_name.clone());
        let mut variant_desc = String::new();
        let mut variant_desc_tokens = 0_usize;

        for variant_attrs in &variant.attrs {
            let description = get_comment_from_attr(variant_attrs);
            if let Some(description) = description {
                variant_desc = description;
                variant_desc_tokens = calculate_token_count(variant_desc.as_str());
            }
        }

        let fields: Vec<_> = variant
            .fields
            .iter()
            .map(|f| {
                // If the field has an identifier (i.e., it is a named field),
                // use it. Otherwise, use the type as the name.
                let field_name = if let Some(ident) = &f.ident {
                    format_ident!("{}", ident)
                } else {
                    format_ident!("{}", to_snake_case(&f.ty.to_token_stream().to_string()))
                };
                let field_type = &f.ty;
                quote! {
                    pub #field_name: #field_type,
                }
            })
            .collect();

        let execute_command_parameters: Vec<_> = variant
            .fields
            .iter()
            .map(|field| {
                let field_name = &field.ident;
                quote! { #field_name: self.#field_name.clone() }
            })
            .collect();

        let number_type = "number";
        let number_ident = format_ident!("{}", number_type);
        let integer_type = "integer";
        let integer_ident = format_ident!("{}", integer_type);
        let string_type = "string";
        let string_ident = format_ident!("{}", string_type);
        let array_type = "array";
        let array_ident = format_ident!("{}", array_type);

        let field_info: Vec<_> = variant
            .fields
            .iter()
            .map(|f| {
                let field_name = if let Some(ident) = &f.ident {
                    format_ident!("{}", ident)
                } else {
                    format_ident!("{}", to_snake_case(&f.ty.to_token_stream().to_string()))
                };
                let field_type = &f.ty;
                match field_type {
                    syn::Type::Path(typepath) if typepath.qself.is_none() => {
                        let type_ident = &typepath.path.segments.last().unwrap().ident;

                        match type_ident.to_string().as_str() {
                            "f32" | "f64" => {
                                return quote! {
                                    generate_value_arg_info!(#number_ident, #field_name)
                                };
                            }
                            "u8" | "u16" | "u32" | "u64" | "u128" | "usize" | "i8" | "i16"
                            | "i32" | "i64" | "i128" | "isize" => {
                                return quote! {
                                    generate_value_arg_info!(#integer_ident, #field_name)
                                };
                            }
                            "String" | "&str" => {
                                return quote! {
                                    generate_value_arg_info!(#string_ident, #field_name)
                                };
                            }
                            _ => {
                                // Not a great way to determine if we've got a struct or an enum,
                                // TODO: remove the panic out of the variant description related
                                // macro so this doesn't blow up.
                                // println!("Field {} is of type {}", field_name, type_ident);
                                return quote! {
                                    generate_enum_info!(#field_type)
                                };
                            }
                        }
                    }
                    syn::Type::Tuple(_) => {
                        println!("Field {} is of tuple type", field_name);
                    }
                    syn::Type::Array(_) => {
                        println!("Field {} is of array type", field_name);
                        return quote! {
                            generate_value_arg_info!(#array_ident, #field_name)
                        };
                    }
                    _ => {}
                }
                quote! {}
            })
            .collect();

        json_generator_functions.push(quote! {
            impl #struct_name {
                pub fn name() -> String {
                    stringify!(#struct_name).to_string()
                }

                pub fn to_function_call() -> ChatCompletionFunctionCall {
                    let function_call_json = json!({
                        "name": stringify!(#struct_name)
                    });

                    ChatCompletionFunctionCall::Object(function_call_json)
                }

                pub fn execute_command(&self) -> #name {
                    #name::#variant_name {
                        #(#execute_command_parameters),*
                    }
                }

                pub fn get_function_json() -> (Value, usize) {
                    let mut parameters = serde_json::Map::new();
                    let mut total_tokens = 0;

                    for (arg_json, arg_tokens) in vec![#(#field_info),*] {
                        total_tokens += arg_tokens;
                        parameters.insert(
                            arg_json.as_object().unwrap().keys().next().unwrap().clone(),
                            arg_json
                                .as_object()
                                .unwrap()
                                .values()
                                .next()
                                .unwrap()
                                .clone(),
                        );
                    }

                    let function_json = json!({
                        "name": stringify!(#struct_name),
                        "description": #variant_desc,
                        "parameters": {
                            "type": "object",
                            "properties": parameters,
                            "required": parameters.keys().collect::<Vec<_>>()
                        }
                    });

                    total_tokens += 12;
                    total_tokens += #variant_desc_tokens;

                    (function_json, total_tokens)
                }
            }
        });

        generated_structs.push(quote! {
            #[derive(serde::Deserialize, Debug)]
            // #[serde(rename_all = "PascalCase")]
            pub struct #struct_name {
                #(#fields)*
            }
        });
    }

    let all_function_calls = quote! {
        pub fn all_function_jsons() -> (serde_json::Value, usize) {
            let results = vec![#(#generated_struct_names::get_function_json(),)*];
            let combined_json = serde_json::Value::Array(results.iter().map(|(json, _)| json.clone()).collect());
            let total_tokens = results.iter().map(|(_, tokens)| tokens).sum();
            (combined_json, total_tokens)
        }
    };

    generated_clap_gpt_enum.push(quote! {
        #[derive(Subcommand)]
        pub enum CommandsGPT {
            GPT { a: String },
        }
    });

    let struct_names: Vec<String> = generated_struct_names
        .iter()
        .map(|name| format!("{}", name))
        .collect();

    let match_arms: Vec<_> = generated_struct_names
        .iter()
        .map(|struct_name| {
            let response_name = format_ident!("{}", struct_name);

            quote! {
                Ok(FunctionResponse::#response_name(response)) => {
                    let result = response.execute_command();
                    return result.run().await;
                }
            }
        })
        .collect();

    let commands_gpt_impl = quote! {
        #[derive(Debug)]
        pub enum FunctionResponse {
            #(
                #generated_struct_names(#generated_struct_names),
            )*
        }

        impl CommandsGPT {
            #all_function_calls

            fn to_snake_case(camel_case: &str) -> String {
                let mut snake_case = String::new();
                for (i, ch) in camel_case.char_indices() {
                    if i > 0 && ch.is_uppercase() {
                        snake_case.push('_');
                    }
                    snake_case.extend(ch.to_lowercase());
                }
                snake_case
            }

            pub fn parse_gpt_function_call(function_call: &FunctionCall) -> Result<FunctionResponse, Box<dyn std::error::Error + Send + Sync + 'static>> {
                match function_call.name.as_str() {
                    #(
                    #struct_names => {
                        match serde_json::from_str::<#generated_struct_names>(&function_call.arguments) {
                            Ok(arguments) => Ok(FunctionResponse::#generated_struct_names(arguments)),
                            Err(_) => {
                                let snake_case_args = function_call.arguments
                                    .as_str()
                                    .split(',')
                                    .map(|s| {
                                        let mut parts = s.split(':');
                                        match (parts.next(), parts.next()) {
                                            (Some(key), Some(value)) => {
                                                let key_trimmed = key.trim_matches(|c: char| !c.is_alphanumeric()).trim();
                                                let key_snake_case = Self::to_snake_case(key_trimmed);
                                                format!("\"{}\":{}", key_snake_case, value)
                                            },
                                            _ => s.to_owned()
                                        }
                                    })
                                    .collect::<Vec<String>>()
                                    .join(",");

                                let snake_case_args = format!("{{{}", snake_case_args);

                                let arguments: #generated_struct_names = serde_json::from_str(&snake_case_args)?;
                                Ok(FunctionResponse::#generated_struct_names(arguments))
                            }
                        }
                    },
                    )*
                    _ => Err(Box::new(CommandError::new("Unknown function name")))
                }
            }

            fn calculate_token_count(text: &str) -> usize {
                let bpe = cl100k_base().unwrap();
                bpe.encode_ordinary(&text).len()
            }

            pub async fn run(
                prompt: &String,
                model_name: &str,
                request_token_limit: usize,
                max_response_tokens: u16,
                custom_system_message: Option<String>,
            ) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync + 'static>> {
                let function_args =
                    get_function_chat_completion_args(CommandsGPT::all_function_jsons)?;
                let mut system_message_tokens = 7;
                let mut system_message = String::from("You are a helpful function calling bot.");
                if let Some(custom_system_message) = custom_system_message {
                    system_message = custom_system_message;
                    system_message_tokens = Self::calculate_token_count(system_message.as_str());
                }

                let request_token_total = function_args.1 + system_message_tokens + Self::calculate_token_count(prompt.as_str());
                if request_token_total > request_token_limit {
                    return Err(Box::new(CommandError::new("Request token count is too high")));
                }

                let request = CreateChatCompletionRequestArgs::default()
                    .max_tokens(max_response_tokens)
                    .model(model_name)
                    .messages([ChatCompletionRequestMessageArgs::default()
                        .role(Role::System)
                        .content(system_message)
                        .build()?,
                    ChatCompletionRequestMessageArgs::default()
                        .role(Role::User)
                        .content(prompt)
                        .build()?])
                    .functions(function_args.0)
                    .function_call("auto")
                    .build()?;

                let client = Client::new();
                let response_message = client
                    .chat()
                    .create(request)
                    .await?
                    .choices
                    .get(0)
                    .unwrap()
                    .message
                    .clone();

                if let Some(function_call) = response_message.function_call {
                    match Self::parse_gpt_function_call(&function_call) {
                        #(#match_arms,)*
                        Ok(_) => Ok(None),
                        Err(e) => {
                            return Err(Box::new(CommandError::new("Something went wrong running gpt command.")));
                        }
                    }
                } else {
                    return Ok(None);
                }
            }
        }
    };

    let gen = quote! {
        #(#generated_structs)*

        #(#json_generator_functions)*

        #(#generated_clap_gpt_enum)*

        #commands_gpt_impl
    };
    // println!("There was an commands:  {:#?}", gen.to_string());

    gen.into()
}

fn get_comment_from_attr(attr: &Attribute) -> Option<String> {
    if attr.path().is_ident("doc") {
        if let Meta::NameValue(meta) = &attr.meta {
            if meta.path.is_ident("doc") {
                let value = meta.value.clone();
                // println!("{:#?}", value);
                match value {
                    Expr::Lit(value) => {
                        // println!("{:#?}", value);
                        match value.lit {
                            Lit::Str(value) => {
                                return Some(value.value());
                            }
                            _ => {
                                return None;
                            }
                        }
                    }
                    _ => {
                        return None;
                    }
                }
            }
        }
    }
    None
}

/// Calculate the token count of a given text string using the Byte Pair Encoding (BPE) tokenizer.
///
/// This function utilizes the BPE tokenizer from the `cl100k_base` library. It tokenizes the given text and
/// returns the count of the tokens. This can be used to measure how many tokens a particular text string
/// consumes, which is often relevant in the context of natural language processing tasks.
///
/// # Arguments
///
/// * `text` - A string slice that holds the text to tokenize.
///
/// # Returns
///
/// * `usize` - The count of tokens in the text.
///
/// # Example
///
/// ```
/// let text = "Hello, world!";
/// let token_count = calculate_token_count(text);
/// println!("Token count: {}", token_count);
/// ```
///
/// Note: This function can fail if the `cl100k_base` tokenizer is not properly initialized or the text cannot be tokenized.
fn calculate_token_count(text: &str) -> usize {
    let bpe = cl100k_base().unwrap();
    bpe.encode_ordinary(text).len()
}

/// Convert a camelCase or PascalCase string into a snake_case string.
///
/// This function iterates over each character in the input string. If the character is an uppercase letter, it adds an
/// underscore before it (except if it's the first character) and then appends the lowercase version of the character
/// to the output string.
///
/// # Arguments
///
/// * `camel_case` - A string slice that holds the camelCase or PascalCase string to convert.
///
/// # Returns
///
/// * `String` - The converted snake_case string.
///
/// # Examplejj
///
/// ```
/// let camel_case = "HelloWorld";
/// let snake_case = to_snake_case(camel_case);
/// assert_eq!(snake_case, "hello_world");
/// ```
fn to_snake_case(camel_case: &str) -> String {
    let mut snake_case = String::new();
    for (i, ch) in camel_case.char_indices() {
        if i > 0 && ch.is_uppercase() {
            snake_case.push('_');
        }
        snake_case.extend(ch.to_lowercase());
    }
    snake_case
}
