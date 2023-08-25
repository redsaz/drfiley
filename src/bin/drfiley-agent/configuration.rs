use config::{Config, ConfigError, Environment, File, FileFormat, ConfigBuilder, builder::DefaultState};
use core::result::Result;
use dotenv::dotenv;
use serde::Deserialize;
use std::env;
use std::path::PathBuf;
use xdg::{BaseDirectories, FileFindIterator};

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

/// Configuration for DrFiley Agent.
///
/// Configuration is loaded in the following order, highest priority to lowest:
///  - environment variables
///  - environment variables in .env file in current working directory
///  - Config file specified by DRFILEY_AGENT_CONFIG environment variable,
///  - Config file in drfiley-agent directory of XDG config directories,
///    example: $HOME/.config/drfiley-agent/agent.toml
///  - Config file drfiley-agent.toml in current working directory
///
/// # Errors
///
/// This [`core::result::Result`] will be an [`Err`] if some IO error occurs
/// during loading or if some required values were not provided
pub fn config() -> Result<Configuration, ConfigError> {
    dotenv().ok(); // Load .env entries into env vars
    let default_config_file = "drfiley-agent.toml";
    let custom_config = env::var("DRFILEY_AGENT_CONFIG").unwrap_or(default_config_file.to_owned());

    let mut builder = Config::builder()
        .add_source(File::new(default_config_file, FileFormat::Toml).required(false));

    if let Ok(base_dirs) = BaseDirectories::with_prefix("drfiley-agent") {
        for xdg_config in base_dirs.find_config_files("agent.toml") {
            builder = builder.add_source(File::new(xdg_config.to_string_lossy().to_string().as_str(), FileFormat::Toml).required(false));
        }
    }

    builder = builder
        .add_source(File::new(custom_config.as_str(), FileFormat::Toml).required(false))
        .add_source(Environment::with_prefix("drfiley_agent"));

    let s = builder
        .set_default("debug", "true")?
        .set_default("max_threads", "1")?
        .set_default("path", ".")?
        .build()?;

    if s.get_bool("debug").unwrap_or_default() {
        eprintln!("custom_config: {custom_config}");
        eprintln!("debug: {:?}", s.get_bool("debug"));
        eprintln!("max_threads: {:?}", s.get_int("max_threads"));
        eprintln!("path: {:?}", s.get::<String>("path"));
    }

    s.try_deserialize()
}
