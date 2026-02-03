pub mod config;
pub mod provider;
pub mod run;
pub mod switch;

pub use config::{AddCommand, ListCommand, RmCommand};
pub use provider::ProviderCommand;
pub use run::RunCommand;
pub use switch::{UseCommand, ResetCommand};

use crate::models::AppConfig;
use anyhow::Result;

pub trait Command {
    fn execute(self, config: &mut AppConfig) -> Result<()>;
}
