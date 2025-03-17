# SubstanceSearch Discord Bot

A Discord bot built in Rust for substance search functionality.

## Project Structure

The project is organized into the following modules:

- `substancesearch-core`: Core bot functionality and structures
- `substancesearch-commands`: Command implementations

## Setup

1. Install Rust and Cargo (if not already installed)
2. Clone this repository
3. Create a Discord application and bot at [Discord Developer Portal](https://discord.com/developers/applications)
4. Configure the bot using one of these methods:

   ### Method 1: Configuration File
   1. Copy `config.example.toml` to `config.toml`
   2. Edit `config.toml` with your bot token and optional guild ID
   ```toml
   [discord]
   token = "your-bot-token-here"
   # Optional: Guild ID for development
   # guild_id = "your-guild-id-here"
   ```

   ### Method 2: Environment Variables
   Set the following environment variables:
   ```bash
   export BOT_DISCORD_TOKEN="your-bot-token"
   # Optional: For guild-specific command registration during development
   export BOT_DISCORD_GUILD_ID="your-guild-id"
   ```

   You can also specify a custom config file path:
   ```bash
   export CONFIG_PATH="/path/to/your/config.toml"
   ```

## Building

```bash
cargo build
```

## Running

```bash
cargo run
```

## Available Commands

- `/help` - Shows information about available commands

## Development

The bot supports both global and guild-specific command registration. For development, it's recommended to use guild-specific commands as they update instantly. Set the guild ID in your configuration to enable guild-specific command registration.

## License

MIT 