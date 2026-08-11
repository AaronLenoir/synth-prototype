use toml::de::Error;

#[derive(Debug)]
pub enum ConfigError {
    TomlError(Error),
    UnknownInstrumentName(String),
}

#[derive(Debug)]
pub enum LoadConfigError {
    Io(std::io::Error),
    Config(ConfigError),
}