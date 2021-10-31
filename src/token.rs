use chrono::{Duration, Utc};
use openidconnect::core::{
    CoreIdTokenFields, CoreJwsSigningAlgorithm, CoreRsaPrivateSigningKey, CoreTokenResponse,
    CoreTokenType,
};
use openidconnect::{
    AccessToken, Audience, EmptyAdditionalClaims, EmptyExtraTokenFields, EndUserUsername, IdToken,
    IdTokenClaims, IssuerUrl, JsonWebKeyId, StandardClaims, SubjectIdentifier,
};
use openidconnect::{AdditionalClaims, TokenResponse};
use rocket::form::Form;
use rocket::State;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::config::Config;

pub struct Tokens {
    pub muted: Arc<Mutex<HashMap<String, String>>>,
}

#[derive(FromForm)]
pub struct PostData {
    pub grant_type: Option<String>,
    pub code: String,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub redirect_uri: String,
}

#[get("/token?<client_id>&<nonce>")]
pub async fn token_endpoint(
    config: &State<Config>,
    client_id: String,
    nonce: Option<String>,
) -> String {
    let token = token(config, client_id, nonce, None, None).await;
    token.id_token().unwrap().to_string()
}

#[post("/token", data = "<post_data>")]
pub async fn post_token_endpoint(
    tokens: &State<Tokens>,
    post_data: Form<PostData>,
) -> String {
    let mutex = tokens.muted.lock().unwrap();
    let token = mutex.get(&post_data.code);
    match token {
        Some(token) => token.to_string(),
        _ => "".to_string(),
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Claims {
    account: String,
    nonce: String,
    signature: String,
}

impl AdditionalClaims for Claims {}

pub async fn token(
    config: &Config,
    client_id: String,
    nonce: Option<String>,
    account: Option<String>,
    signature: Option<String>,
) -> CoreTokenResponse {
    let rsa_pem = config.rsa_pem.clone();

    let _claims = Claims {
        account: account.clone().unwrap_or_default(),
        nonce: nonce.clone().unwrap_or_default(),
        signature: signature.clone().unwrap_or_default(),
    };

    let id_token = IdToken::new(
        IdTokenClaims::new(
            IssuerUrl::new(config.ext_hostname.clone()).unwrap(),
            vec![Audience::new(client_id)],
            // The ID token expiration is usually much shorter than that of the access or refresh
            // tokens issued to clients.
            Utc::now() + Duration::seconds(300),
            // The issue time is usually the current time.
            Utc::now(),
            // Set the standard claims defined by the OpenID Connect Core spec.
            StandardClaims::new(
                // Stable subject identifiers are recommended in place of e-mail addresses or other
                // potentially unstable identifiers. This is the only required claim.
                SubjectIdentifier::new(account.clone().unwrap_or_default()),
            )
            .set_preferred_username(Some(
                EndUserUsername::new(format!(
                    "{};{}{}",
                    signature.unwrap_or_default(),
                    account.unwrap_or_default(),
                    nonce.clone().unwrap_or_default()
                ))
                .into(),
            )),
            // OpenID Connect Providers may supply custom claims by providing a struct that
            // implements the AdditionalClaims trait. This requires manually using the
            // generic IdTokenClaims struct rather than the CoreIdTokenClaims type alias,
            // however.
            EmptyAdditionalClaims {},
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
        None,
        // When returning the ID token alongside an authorization code (e.g., in the implicit
        // flow), it is recommended to pass the authorization code here to set the `c_hash` claim
        // automatically.
        None,
    )
    .unwrap();

    CoreTokenResponse::new(
        AccessToken::new(nonce.unwrap_or_default()),
        CoreTokenType::Bearer,
        CoreIdTokenFields::new(Some(id_token), EmptyExtraTokenFields {}),
    )
}
