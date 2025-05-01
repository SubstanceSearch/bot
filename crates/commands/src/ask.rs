use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::*;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::application::command::CommandOptionType;
use serenity::prelude::*;
use reqwest;
use serde_json::Value;
use substancesearch_core::SettingsKey;
use tracing::{error, info};
use std::time::Duration;

const SYSTEM_PROMPT: &str = r#"You are a harm reduction assistant focused on providing factual, non-judgmental information about substances and harm reduction practices. Your goal is to help people make informed decisions about their health and safety.

Key guidelines:
1. Always provide factual, evidence-based information
2. Never make moral judgments or use stigmatizing language
3. Always include relevant harm reduction advice
4. Cite sources whenever possible (prefer PsychonautWiki, TripSit, Erowid, or scientific papers)
5. Be clear about risks and safety considerations
6. Use neutral, professional language
7. If you don't know something, say so and suggest reliable sources
8. Never encourage or discourage use - focus on providing information
9. Always mention drug checking and testing when relevant
10. Include emergency resources when discussing high-risk situations

Remember: Your role is to provide information, not to make decisions for others."#;

pub fn register() -> CreateApplicationCommand {
    let mut command = CreateApplicationCommand::default();
    command
        .name("ask")
        .description("Ask a question about substance harm reduction")
        .create_option(|option| {
            option
                .name("question")
                .description("Your question about substance harm reduction")
                .kind(CommandOptionType::String)
                .required(true)
        });
    command
}

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) -> Result<(), serenity::Error> {
    // Get the question from the command options
    let question = command.data.options[0].value.as_ref()
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // Start typing indicator and keep it alive until we're done
    let typing = command.channel_id.start_typing(&ctx.http)?;

    // Create HTTP client
    let client = reqwest::Client::new();

    // Get settings and make API call
    let answer = {
        let data = ctx.data.read().await;
        let settings = data.get::<SettingsKey>()
            .expect("Expected Settings in TypeMap")
            .clone();

        // Call Anthropic API
        let response = client.post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &settings.anthropic.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&serde_json::json!({
                "model": "claude-3-opus-20240229",
                "max_tokens": 1000,
                "messages": [{
                    "role": "user",
                    "content": format!("{}\n\nQuestion: {}", SYSTEM_PROMPT, question)
                }]
            }))
            .send()
            .await
            .map_err(|e| serenity::Error::Other("Failed to send request to Anthropic API"))?;

        let response_json: Value = response.json().await
            .map_err(|e| serenity::Error::Other("Failed to parse response from Anthropic API"))?;
        response_json["content"][0]["text"].as_str().unwrap_or("I apologize, but I couldn't generate a response at this time.").to_string()
    };

    // Create and send the response
    command.create_interaction_response(&ctx.http, |response| {
        response.interaction_response_data(|data| {
            data.content(answer)
        })
    }).await?;

    // The typing indicator will automatically stop when `typing` is dropped
    Ok(())
}

pub async fn handle_mention(ctx: &Context, msg: &Message) -> Result<(), serenity::Error> {
    // Start typing indicator and keep it alive until we're done
    let typing = msg.channel_id.start_typing(&ctx.http)?;

    // Create HTTP client
    let client = reqwest::Client::new();

    // Get settings and make API call
    let answer = {
        let data = ctx.data.read().await;
        let settings = data.get::<SettingsKey>()
            .expect("Expected Settings in TypeMap")
            .clone();

        // Call Anthropic API
        let response = client.post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &settings.anthropic.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&serde_json::json!({
                "model": "claude-3-opus-20240229",
                "max_tokens": 1000,
                "messages": [{
                    "role": "user",
                    "content": format!("{}\n\nQuestion: {}", SYSTEM_PROMPT, msg.content)
                }]
            }))
            .send()
            .await
            .map_err(|e| serenity::Error::Other("Failed to send request to Anthropic API"))?;

        let response_json: Value = response.json().await
            .map_err(|e| serenity::Error::Other("Failed to parse response from Anthropic API"))?;
        response_json["content"][0]["text"].as_str().unwrap_or("I apologize, but I couldn't generate a response at this time.").to_string()
    };

    // Create and send the response
    msg.channel_id.send_message(&ctx.http, |m| {
        m.content(answer)
    }).await?;

    // The typing indicator will automatically stop when `typing` is dropped
    Ok(())
} 