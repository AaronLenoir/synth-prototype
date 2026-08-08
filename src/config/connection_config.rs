use serde::Deserialize;

#[derive(Debug, PartialEq, Deserialize)]
pub struct ConnectionConfig {
    pub source: EndPointConfig,
    pub target: EndPointConfig,
}

impl ConnectionConfig {
    pub fn from_connection_tuples(tuples: &Vec<EndPointConfigTuple>) -> Vec<ConnectionConfig> {
        tuples
            .iter()
            .map(|t| ConnectionConfig {
                source: EndPointConfig {
                    instrument: t.0.to_string(),
                    port: t.1.to_string(),
                },
                target: EndPointConfig {
                    instrument: t.2.to_string(),
                    port: t.3.to_string(),
                },
            })
            .collect()
    }
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct EndPointConfig {
    pub instrument: String,
    pub port: String,
}

#[derive(Deserialize, PartialEq, Debug, Default)]
pub struct EndPointConfigTuples {
    pub endpoints: Vec<EndPointConfigTuple>,
}

pub type EndPointConfigTuple = (String, String, String, String);
