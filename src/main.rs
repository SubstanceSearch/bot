use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::model::application::command::Command;
use serenity::model::gateway::Activity;
use serenity::builder::CreateApplicationCommands;
use serenity::async_trait;
use tracing::{error, info};
use substancesearch_core::{Bot, SettingsKey};
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
                    "ask" => commands::ask::run(&ctx, &command).await,
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
                                            let embed = commands::drug::create_substance_embed(&substance, source);
                                            component.create_interaction_response(&ctx.http, |response| {
                                                response.interaction_response_data(|data| {
                                                    data.add_embed(embed)
                                                })
                                            }).await.unwrap_or_else(|e| {
                                                error!("Error sending response: {:?}", e);
                                            });
                                        }
                                        Err(e) => {
                                            component.create_interaction_response(&ctx.http, |response| {
                                                response.interaction_response_data(|data| {
                                                    data.content(format!("Error parsing substance data: {}", e))
                                                })
                                            }).await.unwrap_or_else(|e| {
                                                error!("Error sending response: {:?}", e);
                                            });
                                        }
                                    }
                                }
                                Err(e) => {
                                    component.create_interaction_response(&ctx.http, |response| {
                                        response.interaction_response_data(|data| {
                                            data.content(format!("Error fetching substance data: {}", e))
                                        })
                                    }).await.unwrap_or_else(|e| {
                                        error!("Error sending response: {:?}", e);
                                    });
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    async fn message(&self, ctx: Context, msg: Message) {
        // Check if the bot was mentioned
        if msg.mentions_me(&ctx.http).await.unwrap_or(false) {
            if let Err(why) = commands::ask::handle_mention(&ctx, &msg).await {
                error!("Error handling mention: {:?}", why);
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
        
        // Start status update loop in a separate task
        let ctx_clone = ctx.clone();
        tokio::spawn(async move {
            loop {
                let _guild_count = ctx_clone.cache.guild_count();
                let _user_count: usize = ctx_clone.cache.guilds().iter()
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
    let mut client = Client::builder(&bot.settings.discord.token, GatewayIntents::GUILDS | GatewayIntents::GUILD_MEMBERS | GatewayIntents::GUILD_MESSAGES)
        .event_handler(Handler)
        .application_id(bot.application_id.0)
        .await
        .expect("Error creating client");

    // Store settings in the client's data
    {
        let mut data = client.data.write().await;
        data.insert::<SettingsKey>(bot.settings.clone());
    }

    // Register commands
    let commands = vec![
        commands::help::register(),
        commands::drug::register(),
        commands::ask::register(),
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
