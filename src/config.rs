use std::path::PathBuf;

use clap::Parser;
use serde::{Deserialize, Serialize};

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

    #[clap(flatten)]
    pub futures: FuturesConfig,

    #[clap(flatten)]
    pub resize: ResizeConfig,
}

#[derive(Debug, Clone, Deserialize, Parser)]
pub struct FuturesConfig {
    #[clap(long)]
    pub search_full_text: Option<Status>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, clap::ValueEnum)]
pub enum Status {
    On,
    Off,
}

impl Status {
    pub fn from_u8(value: u8) -> Self {
        match value {
            1 => Status::On,
            0 => Status::Off,
            _ => unimplemented!(),
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            Status::On => 1,
            Status::Off => 0,
        }
    }

    pub fn enabled(&self) -> String {
        match self {
            Status::On => "enabled".to_string(),
            Status::Off => "disabled".to_string(),
        }
    }

    pub fn is_on(&self) -> bool {
        match self {
            Status::On => true,
            Status::Off => false,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Parser)]
pub struct ResizeConfig {
    /// `lanczos3`  
    /// `gaussian`  
    /// `catmull-rom`  
    /// `hamming`  
    /// `mitchell`  
    /// `bilinear`  
    /// `box`  
    #[clap(long = "resize-filter-type", default_value = "lanczos3")]
    pub filter_type: String,

    /// Slow <-
    /// `super-sampling8x`  
    /// `super-sampling4x`  
    /// `super-sampling2x`  
    /// `convolution`  
    /// `interpolation`  
    /// `nearest`  
    /// -> Fast
    /// (nearest will ignore filter_type)
    #[clap(long = "resize-algorithm", default_value = "interpolation")]
    pub algorithm: String,
}
