use crate::claims::{additional_claims, standard_claims, ClaimsMutex};
use crate::config::Config;
use crate::token::{token, Tokens};
use crate::web3::{is_nft_owner_of, validate_signature};
use openidconnect::{AccessToken, AuthorizationCode, TokenResponse};
use rocket::http::Status;
use rocket::response::Redirect;
use rocket::State;
use url::Url;
use uuid::Uuid;

#[get(
    "/<realm>/authorize?<client_id>&<redirect_uri>&<state>&<response_type>&<response_mode>&<nonce>&<account>&<signature>"
)]
pub async fn authorize_endpoint(
    config: &State<Config>,
    claims: &State<ClaimsMutex>,
    tokens: &State<Tokens>,
    realm: String,
    client_id: String,
    redirect_uri: String,
    state: Option<String>,
    response_type: Option<String>,
    response_mode: Option<String>,
    nonce: Option<String>,
    account: Option<String>,
    signature: Option<String>,
) -> Result<Redirect, Status> {
    if account.is_none() {
        let mut url = Url::parse(&format!("{}/{}", config.ext_hostname, realm)).unwrap();
        url.query_pairs_mut()
            .clear()
            .append_pair("client_id", &client_id)
            .append_pair("state", &state.unwrap_or_default())
            .append_pair("nonce", &nonce.unwrap_or_default())
            .append_pair("response_type", &response_type.unwrap_or_default())
            .append_pair("response_mode", &response_mode.unwrap_or_default())
            .append_pair("redirect_uri", &redirect_uri)
            .append_pair("realm", &realm);
        return Ok(Redirect::temporary(url.to_string()));
    };

    if !validate_signature(
        account.clone().unwrap(),
        nonce.clone().unwrap(),
        signature.clone().unwrap(),
    ) {
        return Err(Status::Unauthorized);
    }

    let node_provider = config.node_provider.get(&realm).unwrap();

    if !is_nft_owner_of(
        client_id.clone(),
        account.clone().unwrap_or_default(),
        node_provider.clone(),
    )
    .await
    .unwrap()
    {
        return Err(Status::Unauthorized);
    }

    let access_token = AccessToken::new(Uuid::new_v4().to_string());
    let code = AuthorizationCode::new(Uuid::new_v4().to_string());
    let chain_id = config.chain_id.get(&realm);
    let node = config.node_provider.get(&realm);

    let standard_claims = standard_claims(&account.clone().unwrap());
    let additional_claims = additional_claims(
        &account.unwrap(),
        &nonce.clone().unwrap(),
        &signature.unwrap(),
        chain_id.unwrap(),
        node.unwrap(),
        &client_id,
    );

    claims
        .standard_claims
        .lock()
        .unwrap()
        .insert(access_token.secret().clone(), standard_claims.clone());
    claims
        .additional_claims
        .lock()
        .unwrap()
        .insert(access_token.secret().clone(), additional_claims.clone());

    let mut redirect_uri = Url::parse(&redirect_uri).unwrap();

    let token = token(
        config,
        realm,
        client_id,
        nonce,
        standard_claims,
        additional_claims,
        access_token.clone(),
        code.clone(),
    )
    .await;

    println!("{:?}", access_token.secret());
    println!("{:?}", code.secret());
    let id_token = token.id_token().unwrap().to_string();

    tokens
        .bearer
        .lock()
        .unwrap()
        .insert(code.secret().clone(), access_token.secret().clone());
    tokens
        .muted
        .lock()
        .unwrap()
        .insert(access_token.secret().clone(), token);

    if let Some(response_type) = response_type {
        if response_type.contains("code") {
            redirect_uri
                .query_pairs_mut()
                .append_pair("code", &code.secret());
        }
        if response_type.contains("id_token") {
            redirect_uri
                .query_pairs_mut()
                .append_pair("id_token", &id_token);
        } else if response_type.contains("token") {
            redirect_uri
                .query_pairs_mut()
                .append_pair("id_token", &id_token);
        }
    } else {
        redirect_uri
            .query_pairs_mut()
            .append_pair("code", &code.secret());
    };

    match state {
        Some(state) => {
            redirect_uri.query_pairs_mut().append_pair("state", &state);
        }
        _ => {}
    }

    Ok(Redirect::temporary(redirect_uri.to_string()))
}

#[get(
    "/authorize?<client_id>&<redirect_uri>&<state>&<response_type>&<response_mode>&<nonce>&<account>&<signature>&<realm>"
)]
pub async fn default_authorize_endpoint(
    config: &State<Config>,
    claims: &State<ClaimsMutex>,
    tokens: &State<Tokens>,
    realm: Option<String>,
    client_id: String,
    redirect_uri: String,
    state: Option<String>,
    response_type: Option<String>,
    response_mode: Option<String>,
    nonce: Option<String>,
    account: Option<String>,
    signature: Option<String>,
) -> Result<Redirect, Status> {
    authorize_endpoint(
        config,
        claims,
        tokens,
        realm.unwrap_or("default".into()),
        client_id,
        redirect_uri,
        state,
        response_type,
        response_mode,
        nonce,
        account,
        signature,
    )
    .await
}
