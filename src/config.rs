use std::path::PathBuf;

use clap::Parser;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Deserialize, Parser)]
pub struct Config {
    #[clap(env = "ARCHIVER_PATH", default_value = "archive")]
    pub path: PathBuf,
    #[clap(long, default_value = "3000")]
    pub port: u16,

    #[clap(flatten)]
    pub public: PublicConfig,

    #[clap(flatten)]
    pub futures: FutureConfig,

    #[clap(flatten)]
    pub resize: ResizeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Parser, TS)]
#[ts(export)]
pub struct PublicConfig {
    /// Example: https://static.example.com/archiver
    #[clap(long)]
    pub resource_url: Option<String>,

    /// Example: https://images.example.com/archiver
    #[clap(long)]
    pub images_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Parser)]
pub struct FutureConfig {
    #[cfg(feature = "full-text-search")]
    #[clap(long)]
    pub full_text_search: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Parser)]
pub struct ResizeConfig {
    /// the maximum cache size by number of images
    /// 0 to disable
    #[clap(long = "resize-cache-size", default_value = "200")]
    pub cache_size: usize,

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
