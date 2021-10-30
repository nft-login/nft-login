use crate::config::Config;
use crate::token::token;
use openidconnect::TokenResponse;
use rocket::response::Redirect;
use rocket::State;
use url::Url;
use uuid::Uuid;

#[get(
    "/authorize?<client_id>&<redirect_uri>&<state>&<response_type>&<nonce>&<account>&<signature>"
)]
pub async fn authorize_endpoint(
    config: &State<Config>,
    client_id: String,
    redirect_uri: String,
    state: Option<String>,
    response_type: Option<String>,
    nonce: Option<String>,
    account: Option<String>,
    signature: Option<String>,
) -> Redirect {
    if account.is_none() {
        let mut url = Url::parse(&config.ext_hostname).unwrap();
        url.query_pairs_mut()
            .clear()
            .append_pair("client_id", &client_id)
            .append_pair("state", &state.unwrap_or_default())
            .append_pair("nonce", &nonce.unwrap_or_default())
            .append_pair("response_type", &response_type.unwrap_or_default())
            .append_pair("redirect_uri", &redirect_uri);
        return Redirect::temporary(url.to_string());
    };

    let mut redirect_uri = Url::parse(&redirect_uri).unwrap();

    if let Some(response_type) = response_type {
        if response_type.contains("code") {
            redirect_uri
                .query_pairs_mut()
                .append_pair("code", &Uuid::new_v4().to_string());
        }
        if response_type.contains("id_token") {
            let token = token(config, client_id, nonce, account).await;
            let id_token = token.id_token().unwrap().to_string();
            redirect_uri
                .query_pairs_mut()
                .append_pair("id_token", &id_token);
        } else if response_type.contains("token") {
            let token = token(config, client_id, nonce, account).await;
            let id_token = token.id_token().unwrap().to_string();
            redirect_uri
                .query_pairs_mut()
                .append_pair("id_token", &id_token);
        }
    } else {
        redirect_uri
            .query_pairs_mut()
            .append_pair("token", &Uuid::new_v4().to_string());
    };

    match state {
        Some(state) => {
            redirect_uri.query_pairs_mut().append_pair("state", &state);
        }
        _ => {}
    }

    Redirect::temporary(redirect_uri.to_string())
}
