use machine_launcher_common::{
    Endpoint, GetNonce, GetOidcConfig, ListServers, ServerName, StartServer, StopServer,
};

pub struct Client {
    base_url: String,
    token: Option<String>,
}

impl Client {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            token: None,
        }
    }

    pub fn with_token(mut self, token: String) -> Self {
        self.token = Some(token);
        self
    }

    async fn get<E: Endpoint<Request = ()>>(&self) -> Result<E::Response, reqwest::Error> {
        let mut builder =
            reqwest::Client::new().get(format!("{}{}", self.base_url, E::PATH));
        if let Some(t) = &self.token {
            builder = builder.bearer_auth(t);
        }
        builder.send().await?.json().await
    }

    async fn put<E: Endpoint>(&self, body: &E::Request) -> Result<E::Response, reqwest::Error> {
        let mut builder = reqwest::Client::new()
            .put(format!("{}{}", self.base_url, E::PATH))
            .json(body);
        if let Some(t) = &self.token {
            builder = builder.bearer_auth(t);
        }
        builder.send().await?.json().await
    }

    pub async fn list_servers(&self) -> Result<<ListServers as Endpoint>::Response, reqwest::Error> {
        self.get::<ListServers>().await
    }

    pub async fn start_server(
        &self,
        name: String,
    ) -> Result<<StartServer as Endpoint>::Response, reqwest::Error> {
        self.put::<StartServer>(&ServerName { name }).await
    }

    pub async fn stop_server(
        &self,
        name: String,
    ) -> Result<<StopServer as Endpoint>::Response, reqwest::Error> {
        self.put::<StopServer>(&ServerName { name }).await
    }

    pub async fn get_oidc_config(
        &self,
    ) -> Result<<GetOidcConfig as Endpoint>::Response, reqwest::Error> {
        self.get::<GetOidcConfig>().await
    }

    pub async fn get_nonce(&self) -> Result<<GetNonce as Endpoint>::Response, reqwest::Error> {
        self.get::<GetNonce>().await
    }
}
