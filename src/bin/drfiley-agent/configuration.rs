use config::{Config, ConfigError, Environment, File};
use core::result::Result;
use dotenv::dotenv;
use serde::Deserialize;
use std::env;
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct Configuration {
    pub path: PathBuf,
    pub max_threads: i32,
    pub debug: bool,
    // TODO log and log rotation?
    // TODO db location?
    // TODO partial file read amount for content hash?
    // TODO do full or partial read?
}

/*
drfiley will need to configure:
- priorities for the following tasks:
  - stating all files for the first time
  - stating all files (to detect changes)
  - hashing file and dir content (potential duplicates only)
  - hashing file and dir content (potential duplicate or not)
  - image thumbnails
  - video thumbnails
  - text file indexing

This will probably be done with an array, TBD.
 */

/// Configuration for DrFiley Collector.
///
/// Configuration is loaded in the following order, highest priority to lowest:
///  - environment variables
///  - environment variables in .env file in current working directory
///  - Config file specified by DRFILEY_AGENT_CONFIG environment variable
///  - Config file drfiley-agent.toml in current working directory
///
/// # Errors
///
/// This [`core::result::Result`] will be an [`Err`] if some IO error occurs
/// during loading or if some required values were not provided
pub fn config() -> Result<Configuration, ConfigError> {
    dotenv().ok(); // Load .env entries into env vars
    let config_file = "drfiley-agent.toml";
    let custom_config = env::var("DRFILEY_AGENT_CONFIG").unwrap_or(config_file.to_owned());

    let s = Config::builder()
        .add_source(File::with_name(config_file).required(false))
        .add_source(File::with_name(custom_config.as_str()).required(false))
        .add_source(Environment::with_prefix("drfiley_agent"))
        .set_default("debug", "false")?
        .set_default("max_threads", "1")?
        .set_default("path", ".")?
        .build()?;

    if s.get_bool("debug").unwrap_or_default() {
        println!("debug: {:?}", s.get_bool("debug"));
        println!("max_threads: {:?}", s.get_int("max_threads"));
        println!("path: {:?}", s.get::<String>("path"));
    }

    s.try_deserialize()
}
