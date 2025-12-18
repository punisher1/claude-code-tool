pub mod config;
pub mod provider;
pub mod switch;

pub use config::{AddCommand, ListCommand};
pub use provider::ProviderCommand;
pub use switch::UseCommand;

use crate::models::AppConfig;
use anyhow::Result;

pub trait Command {
    fn execute(self, config: &mut AppConfig) -> Result<()>;
}
