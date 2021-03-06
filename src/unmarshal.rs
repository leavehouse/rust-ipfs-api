use super::{hyper, serde, serde_json};

pub fn unmarshal<'a, T>(bytes: &'a [u8]) -> serde_json::Result<T>
        where T: serde::Deserialize<'a> {
    serde_json::from_slice(bytes)
}

#[derive(Debug, Deserialize)]
pub struct AddInfo {
    pub Name: String,
    pub Hash: String,
    pub Size: String,
}

#[derive(Debug, Deserialize)]
pub struct CommandInfo {
    Name: String,
    Subcommands: Vec<CommandInfo>,
    Options: Vec<CommandNames>,
}

#[derive(Debug, Deserialize)]
pub struct CommandNames {
    Names: Vec<String>
}

#[derive(Debug, Deserialize)]
pub struct ObjectInfo {
    pub Links: Vec<ObjectLinkInfo>,
    pub Data: String,
}

#[derive(Debug, Deserialize)]
pub struct ObjectLinkInfo {
    pub Name: String,
    pub Hash: String,
    pub Size: u64,
}

#[derive(Debug, Deserialize)]
pub struct ObjectLinksInfo {
    pub Hash: String,
    pub Links: Vec<ObjectLinkInfo>,
}

#[derive(Debug, Deserialize)]
pub struct IdInfo {
    ID: String,
    PublicKey: String,
    Addresses: Vec<String>,
    AgentVersion: String,
    ProtocolVersion: String,
}

#[derive(Debug, Deserialize)]
pub struct VersionInfo {
    pub Version: String,
    Commit: String,
    pub Repo: String,
    pub System: String,
    Golang: String,
}
