use chrono::{Duration, Utc};
use openidconnect::core::{
    CoreGenderClaim, CoreJsonWebKeyType, CoreJweContentEncryptionAlgorithm,
    CoreJwsSigningAlgorithm, CoreRsaPrivateSigningKey, CoreTokenType,
};
use openidconnect::{
    AccessToken, Audience, AuthorizationCode, EmptyExtraTokenFields, IdToken, IdTokenClaims,
    IdTokenFields, IssuerUrl, JsonWebKeyId, StandardClaims, StandardTokenResponse,
};
use rocket::form::Form;
use rocket::response::status::NotFound;
use rocket::serde::json::Json;
use rocket::State;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::claims::Claims;
use crate::config::Config;

pub struct Tokens {
    pub muted: Arc<Mutex<HashMap<String, NftTokenResponse>>>,
    pub bearer: Arc<Mutex<HashMap<String, String>>>,
}

#[derive(FromForm)]
pub struct PostData {
    pub grant_type: Option<String>,
    pub code: String,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub redirect_uri: String,
}

#[get("/token?<code>")]
pub async fn default_token_endpoint(
    tokens: &State<Tokens>,
    code: String,
) -> Result<Json<NftTokenResponse>, NotFound<String>> {
    token_endpoint(tokens, "default".into(), code).await
}

#[allow(unused_variables)]
#[get("/<realm>/token?<code>")]
pub async fn token_endpoint(
    tokens: &State<Tokens>,
    realm: String,
    code: String,
) -> Result<Json<NftTokenResponse>, NotFound<String>> {
    let mutex = tokens.bearer.lock().unwrap();
    let access_token = mutex.get(&code).unwrap();
    let mutex = tokens.muted.lock().unwrap();
    let token = mutex.get(access_token);
    match token {
        Some(token) => Ok(Json(token.clone())),
        _ => Err(NotFound("Invalid Code".to_string())),
    }
}

#[post("/token", data = "<post_data>")]
pub async fn default_post_token_endpoint(
    tokens: &State<Tokens>,
    post_data: Form<PostData>,
) -> Result<Json<NftTokenResponse>, NotFound<String>> {
    default_token_endpoint(tokens, post_data.code.clone()).await
}

#[allow(unused_variables)]
#[post("/<realm>/token", data = "<post_data>")]
pub async fn post_token_endpoint(
    tokens: &State<Tokens>,
    realm: String,
    post_data: Form<PostData>,
) -> Result<Json<NftTokenResponse>, NotFound<String>> {
    token_endpoint(tokens, "default".into(), post_data.code.clone()).await
}

pub type NftIdTokenFields = IdTokenFields<
    Claims,
    EmptyExtraTokenFields,
    CoreGenderClaim,
    CoreJweContentEncryptionAlgorithm,
    CoreJwsSigningAlgorithm,
    CoreJsonWebKeyType,
>;

pub type NftTokenResponse = StandardTokenResponse<NftIdTokenFields, CoreTokenType>;

pub async fn token(
    config: &Config,
    client_id: String,
    nonce: Option<String>,
    standard_claims: StandardClaims<CoreGenderClaim>,
    additional_claims: Claims,
    access_token: AccessToken,
    code: AuthorizationCode,
) -> NftTokenResponse {
    let rsa_pem = config.rsa_pem.clone();
    let id_token = IdToken::new(
        IdTokenClaims::new(
            IssuerUrl::new(config.ext_hostname.clone()).unwrap(),
            vec![Audience::new(client_id)],
            Utc::now() + Duration::seconds(300),
            Utc::now(),
            standard_claims,
            additional_claims,
        ),
        // The private key used for signing the ID token. For confidential clients (those able
        // to maintain a client secret), a CoreHmacKey can also be used, in conjunction
        // with one of the CoreJwsSigningAlgorithm::HmacSha* signing algorithms. When using an
        // HMAC-based signing algorithm, the UTF-8 representation of the client secret should
        // be used as the HMAC key.
        &CoreRsaPrivateSigningKey::from_pem(
            &rsa_pem.unwrap_or_default(),
            Some(JsonWebKeyId::new(nonce.clone().unwrap_or_default())),
        )
        .expect("Invalid RSA private key"),
        // Uses the RS256 signature algorithm. This crate supports any RS*, PS*, or HS*
        // signature algorithm.
        CoreJwsSigningAlgorithm::RsaSsaPkcs1V15Sha256,
        // When returning the ID token alongside an access token (e.g., in the Authorization Code
        // flow), it is recommended to pass the access token here to set the `at_hash` claim
        // automatically.
        Some(&access_token),
        // When returning the ID token alongside an authorization code (e.g., in the implicit
        // flow), it is recommended to pass the authorization code here to set the `c_hash` claim
        // automatically.
        Some(&code),
    )
    .unwrap();

    NftTokenResponse::new(
        access_token,
        CoreTokenType::Bearer,
        NftIdTokenFields::new(Some(id_token), EmptyExtraTokenFields {}),
    )
}
