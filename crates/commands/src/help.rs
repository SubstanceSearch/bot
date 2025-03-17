use serenity::prelude::*;
use serenity::builder::CreateApplicationCommand;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;

pub fn register() -> CreateApplicationCommand {
    let mut cmd = CreateApplicationCommand::default();
    cmd.name("help")
        .description("Shows information about available commands");
    cmd
}

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) -> Result<(), SerenityError> {
    command.create_interaction_response(&ctx.http, |response| {
        response.interaction_response_data(|data| {
            data.content("**Available Commands**\n\n\
                `/help` - Shows this help message\n\
                `/drug <name>` - Get information about a substance")
                .ephemeral(true)
        })
    }).await
}
