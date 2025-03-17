use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::model::application::command::Command;
use serenity::model::gateway::Activity;
use serenity::builder::CreateApplicationCommands;
use serenity::async_trait;
use tracing::{error, info};
use substancesearch_core::Bot;
use substancesearch_commands as commands;
use std::time::Duration;
use tokio::time::sleep;
use reqwest;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::ApplicationCommand(command) => {
                let result = match command.data.name.as_str() {
                    "help" => commands::help::run(&ctx, &command).await,
                    "drug" => commands::drug::run(&ctx, &command).await,
                    _ => Ok(()),
                };

                if let Err(why) = result {
                    error!("Error handling command: {:?}", why);
                }
            }
            Interaction::MessageComponent(mut component) => {
                if component.data.custom_id.starts_with("source_") {
                    let source = component.data.custom_id.strip_prefix("source_").unwrap();
                    
                    // Get the original message content to access the substance data
                    if let Some(embed) = component.message.embeds.first() {
                        // Extract substance name from embed title
                        if let Some(title) = &embed.title {
                            // Fetch substance data again
                            let client = reqwest::Client::new();
                            let url = format!("http://1.stg.substancesearch.com/api/substance/{}", title.to_lowercase());
                            
                            match client.get(&url).send().await {
                                Ok(response) => {
                                    match response.json::<commands::drug::SubstanceResponse>().await {
                                        Ok(substance) => {
                                            let new_embed = commands::drug::create_substance_embed(&substance, source);
                                            
                                            // Update the message with the new embed
                                            if let Err(why) = component.message.edit(&ctx.http, |m| {
                                                m.set_embed(new_embed)
                                            }).await {
                                                error!("Error updating embed: {:?}", why);
                                            }
                                        }
                                        Err(e) => {
                                            error!("Error parsing substance data: {}", e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("Error fetching substance data: {}", e);
                                }
                            }
                        }
                    }
                    
                    // Acknowledge the interaction
                    if let Err(why) = component.defer(&ctx.http).await {
                        error!("Error deferring component interaction: {:?}", why);
                    }
                }
            }
            _ => {}
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
        
        // Start status update loop in a separate task
        let ctx_clone = ctx.clone();
        tokio::spawn(async move {
            loop {
                let guild_count = ctx_clone.cache.guild_count();
                let user_count: usize = ctx_clone.cache.guilds().iter()
                    .filter_map(|guild_id| ctx_clone.cache.guild(*guild_id))
                    .map(|guild| guild.member_count as usize)
                    .sum();
                
                let status = "Watching substancesearch.org";
                ctx_clone.set_activity(Activity::watching(&status)).await;
                
                // Update every 5 minutes
                sleep(Duration::from_secs(300)).await;
            }
        });
    }
}

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Initialize bot with config
    let bot = Bot::new().await.expect("Failed to create bot instance");

    // Create client
    let mut client = Client::builder(&bot.settings.discord.token, GatewayIntents::GUILDS | GatewayIntents::GUILD_MEMBERS)
        .event_handler(Handler)
        .application_id(bot.application_id.0)
        .await
        .expect("Error creating client");

    // Register commands
    let commands = vec![
        commands::help::register(),
        commands::drug::register(),
    ];

    // Register either globally or for a specific guild
    if let Some(guild_id) = &bot.settings.discord.guild_id {
        let guild_id = GuildId(guild_id.parse().expect("Invalid guild ID"));
        match guild_id.set_application_commands(&client.cache_and_http.http, |c| {
            *c = CreateApplicationCommands::default();
            c.set_application_commands(commands);
            c
        }).await {
            Ok(_) => info!("Successfully registered guild commands"),
            Err(why) => error!("Failed to register guild commands: {why:?}"),
        }
    } else {
        match Command::set_global_application_commands(&client.cache_and_http.http, |c| {
            *c = CreateApplicationCommands::default();
            c.set_application_commands(commands);
            c
        }).await {
            Ok(_) => info!("Successfully registered global commands"),
            Err(why) => error!("Failed to register global commands: {why:?}"),
        }
    }

    // Start the client
    if let Err(why) = client.start().await {
        error!("Client error: {why:?}");
    }
}
