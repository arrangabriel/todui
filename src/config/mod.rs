mod config;

pub use config::{Config};
use config::RawConfig;

pub fn parse_config(config_str: Option<&str>) -> anyhow::Result<Config> {
    let base_config: Config = toml::from_str(DEFAULT_CONFIG)?;

    let config: Config = if let Some(config_str) = config_str {
        let raw_config = toml::from_str::<RawConfig>(config_str)?;
        base_config.merge(raw_config)
    } else {
        base_config
    };
    Ok(config)
}


static DEFAULT_CONFIG: &str = r#"
[keymap]
delete = "d"

"#;
