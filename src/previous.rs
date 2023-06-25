use log::{info, warn};
use serde::{Deserialize, Serialize};

use crate::error::Error;

#[derive(Serialize, Deserialize, Debug, PartialEq, Default)]
pub struct Previous {
    pub load_cfg_path: Option<String>,
}

impl Previous {
    pub fn write(&self) -> Result<(), Error> {
        let ser_toml = toml::to_string_pretty(&self)?;
        info!("Successfully serialized previous toml file");
        let path = std::env::current_dir()?.join("Cfg").join("previous.toml");
        std::fs::write(path, ser_toml)?;

        Ok(())
    }

    pub fn read_or_default() -> Self {
        let path = std::env::current_dir()
            .unwrap()
            .join("Cfg")
            .join("previous.toml");
        match std::fs::read_to_string(path) {
            Ok(string) => match toml::from_str(&string) {
                Ok(previous) => {
                    info!("Successfully deserialized previous toml file");
                    previous
                }
                Err(e) => {
                    warn!("Failed to deserialize previous toml file. Reason: {e}. Loading default");
                    Self::default()
                }
            },
            Err(e) => {
                warn!("Failed to open previous toml file. Reason: {e}. Loading default");
                Self::default()
            }
        }
    }
}
