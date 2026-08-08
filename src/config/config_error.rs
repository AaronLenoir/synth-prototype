use toml::de::Error;

#[derive(Debug)]
pub enum ConfigError {
    Unknown,
    TomlError(Error),
}
