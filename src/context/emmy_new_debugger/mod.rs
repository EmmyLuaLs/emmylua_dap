use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmmyNewDebugArguments {
    pub host: String,
    pub port: u16,
    pub ext: Vec<String>,
    pub ide_connect_debugger: bool,
}
