pub mod config;
pub mod provider;
pub mod run;
pub mod switch;
pub mod update;

pub use config::{AddCommand, ListCommand, RmCommand};
pub use provider::ProviderCommand;
pub use run::RunCommand;
pub use switch::{ResetCommand, UseCommand};
pub use update::UpdateCommand;

use crate::models::AppConfig;
use anyhow::Result;

pub trait Command {
    fn execute(self, config: &mut AppConfig) -> Result<()>;
}
