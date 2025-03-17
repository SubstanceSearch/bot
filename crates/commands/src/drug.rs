use serenity::builder::{CreateApplicationCommand, CreateEmbed, CreateButton};
use serenity::model::prelude::*;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::application::command::CommandOptionType;
use serenity::model::application::component::ButtonStyle;
use serenity::prelude::*;
use serde::Deserialize;
use serde_json::Value;
use reqwest;

#[derive(Deserialize)]
pub struct SubstanceResponse {
    pub psychonautwiki: PsychonautWikiData,
    pub tripsit: TripSitData,
}

#[derive(Deserialize)]
pub struct PsychonautWikiData {
    pub pretty_name: String,
    pub aliases: Vec<String>,
    pub effects: Vec<String>,
    pub dosage: DosageInfo,
    pub timing: TimingInfo,
}

#[derive(Deserialize)]
pub struct TripSitData {
    pub pretty_name: String,
    pub aliases: Vec<String>,
    pub effects: Vec<String>,
    pub dosage: DosageInfo,
    pub timing: TimingInfo,
    pub properties: Properties,
}

#[derive(Deserialize)]
pub struct DosageInfo {
    pub routes: Option<std::collections::HashMap<String, RouteInfo>>,
}

#[derive(Deserialize)]
pub struct RouteInfo {
    pub common: Option<DoseRange>,
    pub light: Option<DoseRange>,
    pub strong: Option<DoseRange>,
    pub units: Option<String>,
}

#[derive(Deserialize)]
pub struct DoseRange {
    pub min: f32,
    pub max: f32,
}

#[derive(Deserialize)]
pub struct TimingInfo {
    pub onset: TimeValue,
    pub duration: TimeValue,
    pub aftereffects: TimeValue,
}

#[derive(Deserialize)]
pub struct TimeValue {
    pub value: Option<TimeValueInner>,
    #[serde(default)]
    pub sublingual: Option<TimeValueInner>,
}

#[derive(Deserialize)]
pub struct TimeValueInner {
    pub value: String,
    pub unit: String,
}

#[derive(Deserialize)]
pub struct Properties {
    pub summary: String,
}

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) -> Result<(), serenity::Error> {
    let substance_name = command.data.options.get(0)
        .and_then(|opt| opt.value.as_ref().and_then(|v| v.as_str()))
        .unwrap_or("unknown");

    // Defer the response while we fetch the data
    command.defer(&ctx.http).await?;

    // Fetch substance data
    let client = reqwest::Client::new();
    let url = format!("http://1.stg.substancesearch.com/api/substance/{}", substance_name.to_lowercase());
    
    match client.get(&url).send().await {
        Ok(response) => {
            match response.json::<SubstanceResponse>().await {
                Ok(substance) => {
                    // Create initial embed with TripSit data
                    let embed = create_substance_embed(&substance, "tripsit");
                    
                    // Create source selection buttons
                    let buttons = create_source_buttons();
                    
                    // Send the response with the embed and buttons
                    command.edit_original_interaction_response(&ctx.http, |response| {
                        response
                            .add_embed(embed)
                            .components(|c| {
                                c.create_action_row(|row| {
                                    row.add_button(buttons.0)
                                        .add_button(buttons.1)
                                })
                            })
                    }).await?;
                }
                Err(e) => {
                    command.edit_original_interaction_response(&ctx.http, |response| {
                        response.content(format!("Error parsing substance data: {}", e))
                    }).await?;
                }
            }
        }
        Err(e) => {
            command.edit_original_interaction_response(&ctx.http, |response| {
                response.content(format!("Error fetching substance data: {}", e))
            }).await?;
        }
    }

    Ok(())
}

pub fn create_substance_embed(substance: &SubstanceResponse, source: &str) -> CreateEmbed {
    let mut embed = CreateEmbed::default();
    
    match source {
        "psychonautwiki" => {
            let data = &substance.psychonautwiki;
            embed.title(&data.pretty_name)
                .description(format!("**Aliases**: {}", data.aliases.join(", ")))
                .color(0x7289DA);

            // Add dosage information if available
            if let Some(routes) = &data.dosage.routes {
                if let Some(oral) = routes.get("oral").or(routes.get("sublingual")) {
                    let mut dosage_text = String::new();
                    if let Some(light) = &oral.light {
                        dosage_text.push_str(&format!("Light: {}-{}", light.min, light.max));
                    }
                    if let Some(common) = &oral.common {
                        dosage_text.push_str(&format!("\nCommon: {}-{}", common.min, common.max));
                    }
                    if let Some(strong) = &oral.strong {
                        dosage_text.push_str(&format!("\nStrong: {}-{}", strong.min, strong.max));
                    }
                    if let Some(units) = &oral.units {
                        dosage_text.push_str(&format!(" {}", units));
                    }
                    
                    embed.field("Dosage", dosage_text, true);
                }
            }

            // Add timing information
            let timing = format!(
                "Onset: {}\nDuration: {}\nAfterglow: {}",
                data.timing.onset.sublingual.as_ref().map_or("Unknown".to_string(), |t| format!("{} {}", t.value, t.unit)),
                data.timing.duration.sublingual.as_ref().map_or("Unknown".to_string(), |t| format!("{} {}", t.value, t.unit)),
                data.timing.aftereffects.sublingual.as_ref().map_or("Unknown".to_string(), |t| format!("{} {}", t.value, t.unit))
            );
            embed.field("Timing", timing, true);

            // Add effects (limited to first 1024 characters due to Discord limits)
            let effects = data.effects.iter()
                .take(15)
                .map(|s| format!("• {}", s))
                .collect::<Vec<_>>()
                .join("\n");
            embed.field("Common Effects", effects, false);
        }
        "tripsit" => {
            let data = &substance.tripsit;
            embed.title(&data.pretty_name)
                .description(format!("**Aliases**: {}\n\n{}", 
                    data.aliases.join(", "),
                    data.properties.summary))
                .color(0xFF4500);

            // Add dosage information if available
            if let Some(routes) = &data.dosage.routes {
                if let Some(oral) = routes.get("oral") {
                    let mut dosage_text = String::new();
                    if let Some(light) = &oral.light {
                        dosage_text.push_str(&format!("Light: {}-{}", light.min, light.max));
                    }
                    if let Some(common) = &oral.common {
                        dosage_text.push_str(&format!("\nCommon: {}-{}", common.min, common.max));
                    }
                    if let Some(strong) = &oral.strong {
                        dosage_text.push_str(&format!("\nStrong: {}-{}", strong.min, strong.max));
                    }
                    if let Some(units) = &oral.units {
                        dosage_text.push_str(&format!(" {}", units));
                    }
                    
                    embed.field("Dosage", dosage_text, true);
                }
            }

            // Add timing information
            let timing = format!(
                "Onset: {}\nDuration: {}\nAfterglow: {}",
                data.timing.onset.value.as_ref().map_or("Unknown".to_string(), |t| format!("{} {}", t.value, t.unit)),
                data.timing.duration.value.as_ref().map_or("Unknown".to_string(), |t| format!("{} {}", t.value, t.unit)),
                data.timing.aftereffects.value.as_ref().map_or("Unknown".to_string(), |t| format!("{} {}", t.value, t.unit))
            );
            embed.field("Timing", timing, true);

            // Add effects
            let effects = data.effects.iter()
                .take(15)
                .map(|s| format!("• {}", s))
                .collect::<Vec<_>>()
                .join("\n");
            embed.field("Common Effects", effects, false);
        }
        _ => {}
    }

    embed.footer(|f| f.text(format!("Source: {}", source)));
    embed
}

fn create_source_buttons() -> (CreateButton, CreateButton) {
    let mut psychonautwiki_btn = CreateButton::default();
    psychonautwiki_btn
        .custom_id("source_psychonautwiki")
        .label("PsychonautWiki")
        .style(ButtonStyle::Secondary);

    let mut tripsit_btn = CreateButton::default();
    tripsit_btn
        .custom_id("source_tripsit")
        .label("TripSit")
        .style(ButtonStyle::Primary);

    (psychonautwiki_btn, tripsit_btn)
}

pub fn register() -> CreateApplicationCommand {
    let mut command = CreateApplicationCommand::default();
    command
        .name("drug")
        .description("Get information about a substance")
        .create_option(|option| {
            option
                .name("name")
                .description("The name of the substance")
                .kind(CommandOptionType::String)
                .required(true)
        });
    command
} 