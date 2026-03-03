use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct SiteConfig {
    pub title:       Option<String>,
    pub description: Option<String>,
    pub author:      Option<String>,
    pub base_url:    Option<String>,
    pub content_dir: Option<String>,
    pub output_dir:  Option<String>,
    pub port:        Option<u16>,
}

impl SiteConfig {
    pub fn load(project_dir: &Path) -> Result<Self> {
        let path = project_dir.join("arcadia.toml");
        if !path.exists() {
            return Ok(Self::default());
        }
        let s = fs::read_to_string(&path).context("read arcadia.toml")?;
        toml::from_str(&s).context("parse arcadia.toml")
    }
}
