mod claude_adapter;
mod commands;
mod config_manager;
mod models;
mod provider_store;
mod utils;

use anyhow::Result;
use clap::{Parser, Subcommand};
use commands::{Command, config::AddCommand, provider::ProviderCommand, switch::UseCommand};
use config_manager::ConfigManager;

#[derive(Parser)]
#[command(name = "cct")]
#[command(about = "Claude Code Tool - Manage API providers for claudecode")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage providers
    #[command(subcommand)]
    Provider(ProviderSubCommand),

    /// Add a new configuration
    Add {
        /// Configuration alias
        alias: String,

        /// Provider name
        #[arg(short, long)]
        provider: String,

        /// API key
        #[arg(short, long)]
        api_key: String,
    },

    /// Use a configuration
    Use {
        /// Configuration alias (optional, will show interactive selection if not provided)
        alias: Option<String>,
    },
}

#[derive(Subcommand)]
enum ProviderSubCommand {
    /// List all available providers
    #[command(alias = "ls")]
    List,

    /// Add a custom provider
    Add {
        /// Provider name (optional, will prompt if not provided)
        name: Option<String>,
    },

    /// Remove a custom provider
    Rm {
        /// Provider name
        name: String,
    },
}

fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();

    // Load or create config
    let mut config = ConfigManager::load_config()?;

    match cli.command {
        Commands::Provider(subcommand) => {
            let cmd = match subcommand {
                ProviderSubCommand::List => ProviderCommand::List,
                ProviderSubCommand::Add { name } => ProviderCommand::Add { name },
                ProviderSubCommand::Rm { name } => ProviderCommand::Remove { name },
            };
            cmd.execute(&mut config)?;
        }
        Commands::Add {
            alias,
            provider,
            api_key,
        } => {
            let cmd = AddCommand {
                alias,
                provider,
                api_key,
            };
            cmd.execute(&mut config)?;
        }
        Commands::Use { alias } => {
            let cmd = UseCommand { alias };
            cmd.execute(&mut config)?;
        }
    }

    Ok(())
}
