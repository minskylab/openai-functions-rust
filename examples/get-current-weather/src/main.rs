use async_openai::{
    types::{
        ChatCompletionFunctionCall, ChatCompletionRequestMessageArgs,
        CreateChatCompletionRequestArgs, Role,
    },
    Client,
};
use openai_func_enums::{
    arg_description, func_description, generate_enum_info, get_function_chat_completion_args,
    parse_function_call, EnumDescriptor, FunctionCallResponse, VariantDescriptors,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();

    let client = Client::new();

    let function_args =
        get_function_chat_completion_args(GetCurrentWeatherResponse::get_function_json)?;
    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(512u16)
        .model("gpt-3.5-turbo-0613")
        .messages([ChatCompletionRequestMessageArgs::default()
            .role(Role::User)
            .content("What's the weather like in Swainsboro, Georgia?")
            .build()?])
        // .functions(vec![function_args.0])
        .functions(function_args.0)
        .function_call(GetCurrentWeatherResponse::to_function_call())
        .build()?;

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
        println!("This is the function call returned:");
        println!("{:#?}", function_call);

        let current_weather_response =
            parse_function_call!(function_call, GetCurrentWeatherResponse);

        if let Some(current_weather_response) = current_weather_response {
            match current_weather_response.location {
                Location::Atlanta => {
                    println!("Function called with location: Atlanta");
                }
                _ => {
                    println!("Function call with a location other than Atlanta.");
                }
            }
        }
    }

    Ok(())
}

#[derive(Debug, FunctionCallResponse)]
pub enum FunctionDef {
    #[func_description(
        description = "Get the current weather in the location closest to the one provided location"
    )]
    GetCurrentWeather(Location, TemperatureUnits),
}

#[derive(Clone, Debug, Deserialize, EnumDescriptor, VariantDescriptors)]
#[arg_description(description = "The only valid locations that can be passed.")]
pub enum Location {
    Atlanta,
    Boston,
    Chicago,
    Dallas,
    Denver,
    LosAngeles,
    Miami,
    Nashville,
    NewYork,
    Philadelphia,
    Seattle,
    StLouis,
    Washington,
}

#[derive(Clone, Debug, Deserialize, EnumDescriptor, VariantDescriptors)]
#[arg_description(description = "A temperature unit chosen from the enum.")]
pub enum TemperatureUnits {
    Celcius,
    Fahrenheit,
}
