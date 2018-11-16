/* This file is part of the Aeterno init system. */

use std::path::PathBuf;

#[cfg(feature = "native")]
const DEFAULT_MASTER_CONFIG: &str = "/etc/aeterno/master.toml";

#[cfg(feature = "default")]
const DEFAULT_MASTER_CONFIG: &str = "./samples/master.toml";

#[derive(Debug)]
pub struct MasterConfiguration {
    pub config_path: PathBuf,
}

pub fn parse_config() -> MasterConfiguration {
    MasterConfiguration {
        config_path: DEFAULT_MASTER_CONFIG.into(),
    }
}
