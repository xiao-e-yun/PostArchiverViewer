use std::path::PathBuf;

use clap::Parser;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, Parser)]
pub struct Config {
    #[clap(env = "ARCHIVER_PATH", default_value = "archiver")]
    pub path: PathBuf,
    /// Example: https://static.example.com/archiver
    #[clap(long)]
    pub resource_url: Option<String>,
    /// Example: https://images.example.com/archiver
    #[clap(long)]
    pub images_url: Option<String>,
    #[clap(long, default_value = "3000")]
    pub port: u16,
}
