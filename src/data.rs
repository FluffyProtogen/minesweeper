use std::{fs::File, sync::Arc};

use serde::{Deserialize, Serialize};
use std::io::BufReader;

#[derive(Serialize, Deserialize, Debug)]
pub struct AppConfiguration {
    pub token: String,
}

pub struct ConfigKey;

impl serenity::prelude::TypeMapKey for ConfigKey {
    type Value = Arc<AppConfiguration>;
}

pub fn load_configuration() -> Result<AppConfiguration, Box<dyn std::error::Error>> {
    let config_path = std::env::current_exe()?.with_file_name("config.json");

    let file = File::open(config_path)?;
    let reader = BufReader::new(file);

    let mut deserializer = serde_json::Deserializer::from_reader(reader);

    let config = AppConfiguration::deserialize(&mut deserializer)?;

    Ok(config)
}
