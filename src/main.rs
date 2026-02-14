mod claude_adapter;
mod commands;
mod config_manager;
mod models;
mod provider_store;
mod utils;

use anyhow::Result;
use clap::{Parser, Subcommand};
use commands::{Command, AddCommand, ListCommand, ProviderCommand, UseCommand, ResetCommand, RmCommand, RunCommand};
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

        /// API key (optional, can be set later via environment variables)
        #[arg(short, long)]
        api_key: Option<String>,

        /// HTTP/HTTPS proxy URL (e.g., http://proxy.example.com:8080)
        #[arg(long)]
        proxy: Option<String>,
    },

    /// List all configurations
    #[command(alias = "ls")]
    List,

    /// Use a configuration
    Use {
        /// Configuration alias (optional, will show interactive selection if not provided)
        alias: Option<String>,
    },

    /// Clear all provider settings
    Reset,

    /// Remove a configuration
    Rm {
        /// Configuration alias
        alias: String,
    },

    /// Run Claude Code with a configuration
    Run {
        /// Configuration alias
        alias: String,

        /// HTTP/HTTPS proxy URL (e.g., http://127.0.0.1:11225)
        #[arg(long)]
        proxy: Option<String>,

        /// Arguments to pass to claude (after --)
        #[arg(last = true, allow_hyphen_values = true)]
        claude_args: Vec<String>,
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

    /// Export built-in providers to ~/.cct/providers.toml
    Init,
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
                ProviderSubCommand::Init => ProviderCommand::Init,
            };
            cmd.execute(&mut config)?;
        }
        Commands::Add {
            alias,
            provider,
            api_key,
            proxy,
        } => {
            let cmd = AddCommand {
                alias,
                provider,
                api_key,
                proxy,
            };
            cmd.execute(&mut config)?;
        }
        Commands::List => {
            let cmd = ListCommand;
            cmd.execute(&mut config)?;
        }
        Commands::Use { alias } => {
            let cmd = UseCommand { alias };
            cmd.execute(&mut config)?;
        }
        Commands::Reset => {
            let cmd = ResetCommand;
            cmd.execute(&mut config)?;
        }
        Commands::Rm { alias } => {
            let cmd = RmCommand { alias };
            cmd.execute(&mut config)?;
        }
        Commands::Run { alias, proxy, claude_args } => {
            let cmd = RunCommand { alias, proxy, claude_args };
            cmd.execute(&mut config)?;
        }
    }

    Ok(())
}
