use serenity::builder::CreateApplicationCommand;

pub mod help;
pub mod drug;
pub mod ask;

pub use help::register as register_help;
pub use drug::register as register_drug;
pub use ask::register as register_ask;

pub fn register_commands() -> Vec<CreateApplicationCommand> {
    vec![
        help::register(),
        drug::register(),
        ask::register(),
    ]
}
