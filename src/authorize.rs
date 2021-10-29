use crate::config::Config;
use crate::token::token;
use openidconnect::TokenResponse;
use rocket::response::Redirect;
use rocket::State;
use url::Url;
use uuid::Uuid;

#[get("/authorize?<redirect_uri>&<state>&<response_type>")]
pub async fn authorize_endpoint(
    config: &State<Config>,
    redirect_uri: String,
    state: Option<String>,
    response_type: Option<String>,
) -> Redirect {
    let mut redirect_uri = Url::parse(&redirect_uri).unwrap();

    if let Some(response_type) = response_type {
        if response_type.contains("code") {
            redirect_uri
                .query_pairs_mut()
                .append_pair("code", &Uuid::new_v4().to_string());
        }
        if response_type.contains("id_token") {
            let token = token(config).await;
            let id_token = token.id_token().unwrap().to_string();
            redirect_uri
                .query_pairs_mut()
                .append_pair("id_token", &id_token);
        } else if response_type.contains("token") {
            let token = token(config).await;
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
