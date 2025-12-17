pub mod config;
pub mod provider;
pub mod switch;

use crate::models::AppConfig;
use anyhow::Result;

pub trait Command {
    fn execute(self, config: &mut AppConfig) -> Result<()>;
}
