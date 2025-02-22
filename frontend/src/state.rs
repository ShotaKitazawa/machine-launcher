#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct Userinfo {
    pub name: String,
    pub icon_url: String,
}

pub type Server = openapi::models::Server;
