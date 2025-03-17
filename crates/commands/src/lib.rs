use serenity::builder::CreateApplicationCommand;

pub mod help;
pub mod drug;

pub use help::register as register_help;
pub use drug::register as register_drug;

pub fn register_commands() -> Vec<CreateApplicationCommand> {
    vec![
        help::register(),
    ]
}
