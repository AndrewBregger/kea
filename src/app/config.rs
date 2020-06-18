extern crate serde;
extern crate serde_yaml;

use crate::font::{FontDesc, Properties};
use std::collections::BTreeMap;
use std::io::Read;

#[derive(Debug, Clone)]
pub struct FontConfig {
    family: String,
    size: f32,
}

/// user defined and default settings to be used by the editor
/// Used to set: theme, font, font size, key bindings
#[derive(Debug, Clone)]
pub struct Config {
    font: FontConfig,
    theme_scheme: String,
}

impl Config {
    pub fn load_config() -> Self {
        let mut f = std::fs::File::open("./config/kea.yml").unwrap();
        let mut content = String::new();
        f.read_to_string(&mut content).unwrap();
        let config: BTreeMap<String, BTreeMap<String, String>> =
            serde_yaml::from_str(&content).unwrap();
        let font_name = config["font"]["family"].as_str();
        let size = config["font"]["size"]
            .as_str()
            .parse::<f32>()
            .unwrap_or(14f32);
        Self {
            font: FontConfig {
                family: font_name.to_string(),
                size,
            },
            theme_scheme: "".to_string(),
        }
    }

    pub fn font_name(&self) -> &str {
        self.font.family.as_str()
    }

    pub fn font_desc(&self) -> FontDesc {
        FontDesc::new(self.font.family.as_str(), Properties::new())
    }

    pub fn font_size(&self) -> f32 {
        self.font.size
    }
}
