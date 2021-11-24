use async_graphql::*;
use serde::Deserialize;

pub(crate) struct Auth;

#[Object]
impl Auth {
    async fn github_login(&self, code: String) -> Result<LoginResponse> {
        let client_id = std::env::var("KAMU_AUTH_GITHUB_CLIENT_ID")
            .expect("KAMU_AUTH_GITHUB_CLIENT_ID env var is not set");
        let client_secret = std::env::var("KAMU_AUTH_GITHUB_CLIENT_SECRET")
            .expect("KAMU_AUTH_GITHUB_CLIENT_SECRET env var is not set");

        let params = [
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("code", code),
        ];

        let client = reqwest::blocking::Client::builder()
            .user_agent(concat!(
                env!("CARGO_PKG_NAME"),
                "/",
                env!("CARGO_PKG_VERSION"),
            ))
            .build()?;

        let body = client
            .post("https://github.com/login/oauth/access_token")
            .header(reqwest::header::ACCEPT, "application/json")
            .form(&params)
            .send()?
            .error_for_status()?
            .text()?;

        let token = serde_json::from_str::<AccessToken>(&body)
            .unwrap_or_else(|_| panic!("Failed auth with error: {:?}", body));

        let account_info = client
            .get("https://api.github.com/user")
            .bearer_auth(&token.access_token)
            .header(reqwest::header::ACCEPT, "application/vnd.github.v3+json")
            .send()?
            .error_for_status()?
            .json::<AccountInfo>()?;

        Ok(LoginResponse {
            token,
            account_info,
        })
    }
}

#[derive(SimpleObject, Debug, Clone, Deserialize)]
pub(crate) struct LoginResponse {
    token: AccessToken,
    account_info: AccountInfo,
}

#[derive(SimpleObject, Debug, Clone, Deserialize)]
pub(crate) struct AccessToken {
    access_token: String,
    scope: String,
    token_type: String,
}

#[derive(SimpleObject, Debug, Clone, Deserialize)]
pub(crate) struct AccountInfo {
    login: String,
    avatar_url: String,
    gravatar_id: String,
    name: String,
    email: String,
}
