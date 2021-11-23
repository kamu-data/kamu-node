use async_graphql::*;
use serde::Deserialize;

pub(crate) struct Auth;

#[Object]
impl Auth {
    async fn github_login(&self, code: String) -> Result<OAuthLoginResponse> {
        let client_id = std::env::var("KAMU_AUTH_GITHUB_CLIENT_ID")
            .expect("KAMU_AUTH_GITHUB_CLIENT_ID env var is not set");
        let client_secret = std::env::var("KAMU_AUTH_GITHUB_CLIENT_SECRET")
            .expect("KAMU_AUTH_GITHUB_CLIENT_SECRET env var is not set");

        let params = [
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("code", code),
        ];

        let client = reqwest::blocking::Client::new();
        let body = client
            .post("https://github.com/login/oauth/access_token")
            .header("Accept", "application/json")
            .form(&params)
            .send()?
            .error_for_status()?
            .text()?;

        match serde_json::from_str::<OAuthLoginResponse>(&body) {
            Ok(token) => Ok(token),
            Err(_) => panic!("Failed auth with error: {:?}", body),
        }
    }
}

#[derive(SimpleObject, Debug, Clone, Deserialize)]
pub(crate) struct OAuthLoginResponse {
    access_token: String,
    scope: String,
    token_type: String,
}
