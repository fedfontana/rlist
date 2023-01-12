use std::path::PathBuf;

use serde::Deserialize;


#[derive(Deserialize, Debug)]
pub struct Config {
    pub color: Option<bool>,
    pub db_file: Option<PathBuf>,
    pub datetime_format: Option<String>,
    pub print_counts: Option<bool>,
    pub always_long_list: Option<bool>,
}