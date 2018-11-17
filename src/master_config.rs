/* This file is part of the Aeterno init system. */

use std::io::Read;
use std::fs::File;
use std::path::PathBuf;

use nix::Result;

#[cfg(feature = "native")]
const DEFAULT_MASTER_CONFIG: &str = "/etc/aeterno/master.toml";
#[cfg(feature = "default")]
const DEFAULT_MASTER_CONFIG: &str = "./samples/master.toml";

#[derive(Deserialize, Debug)]
pub struct MasterConfiguration {
    pub slaves: Vec<PathBuf>,
}

pub fn parse_config() -> Result<MasterConfiguration> {
        let mut configfile = File::open(DEFAULT_MASTER_CONFIG)
			.or(Err(nix::Error::Sys(nix::errno::Errno::EINVAL)))?;

        let mut cfile_cts = String::new();
        configfile.read_to_string(&mut cfile_cts)
			.or(Err(nix::Error::Sys(nix::errno::Errno::ENOENT)))?;

        let cfg: MasterConfiguration = toml::from_str(&cfile_cts).unwrap();

		Ok(cfg)
}
