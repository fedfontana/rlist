use anyhow::{Context, Result};
use colored::Colorize;
use serde::Deserialize;
use std::{
    env,
    path::{Path, PathBuf},
};

use crate::utils::format_string_is_valid;

#[derive(Deserialize, Debug)]
pub struct ConfigContent {
    pub db_file: Option<PathBuf>,
    pub datetime_format: Option<String>,
}

pub struct Config {
    pub db_file: PathBuf,
    pub datetime_format: String,
}

const DEFAULT_DATETIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

impl Config {
    fn maybe_default() -> Result<Self> {
        Ok(Self {
            db_file: get_default_db_file_path()?.into(),
            datetime_format: DEFAULT_DATETIME_FORMAT.to_string(),
        })
    }
}

fn get_default_db_file_path() -> Result<String> {
    let home_dir_path = dirs::home_dir().ok_or(anyhow::anyhow!("Could not find home folder"))?;
    let rlist_dir = Path::new(home_dir_path.as_os_str()).join("rlist");
    Ok(rlist_dir
        .join("rlist.sqlite")
        .to_str()
        .ok_or(anyhow::anyhow!(
            "Could not get the default reading list location"
        ))?
        .to_string())
}

fn get_default_config_file_path() -> Result<String> {
    let config_dir_path = if env::consts::OS == "macos" {
        dirs::home_dir()
            .ok_or(anyhow::anyhow!("Could not find config folder"))?
            .join(".config")
    } else {
        dirs::config_dir().ok_or(anyhow::anyhow!("Could not find config folder"))?
    };

    let default_config_path = Path::new(config_dir_path.as_os_str()).join("rlist.yml");
    Ok(default_config_path
        .to_str()
        .ok_or(anyhow::anyhow!(
            "Could not get the default rlist config location"
        ))?
        .to_string())
}

impl Config {
    /// Prints warnings
    pub fn new_from_content(content: ConfigContent) -> Result<Self> {
        let format = content.datetime_format.map(|f| {
            if format_string_is_valid(f.as_str()) {
                f
            } else {
                eprintln!("{}: the datetime format provided in your custom config is not a valid format string, reverting to the default datetime representation.", "Warning".bold().yellow());
                eprintln!("{}: Please refer to https://docs.rs/chrono/latest/chrono/format/strftime/index.html for the available formatting options\n", "Info".bold().cyan());
                DEFAULT_DATETIME_FORMAT.to_string()
            }
        }).unwrap_or(DEFAULT_DATETIME_FORMAT.to_string());

        let db_file_path = if let Some(p) = content.db_file {
            let path = Path::new(&p);
            if path.is_relative() {
                return Err(anyhow::anyhow!("The db_file config option must contain an absolute path to the desired reading list location"));
            }
            p
        } else {
            get_default_db_file_path()?.into()
        };

        Ok(Self {
            db_file: db_file_path,
            datetime_format: format,
        })
    }

    pub fn new_from_arg(opt_path: Option<PathBuf>) -> Result<Self> {
        match opt_path {
            // If a custom config path is provided, then read it
            Some(p) => {
                let file_content =
                    std::fs::read_to_string(p).context("Could not read rlist config file")?;
                let config_content: ConfigContent = serde_yaml::from_str(&file_content)?;
                Ok(Self::new_from_content(config_content)?)
            }
            None => {
                // Else, if no custom path is provided look in the default location.
                let default_config_path = get_default_config_file_path()?;
                let config = if Path::new(&default_config_path).exists() {
                    let config_data = std::fs::read_to_string(default_config_path)
                        .context("Could not read rlist config file")?;

                    let config_content: ConfigContent = serde_yaml::from_str(&config_data)?;

                    Self::new_from_content(config_content)?
                } else {
                    // If no file is found in the default location, then use defaults
                    Self::maybe_default()?
                };
                Ok(config)
            }
        }
    }
}
