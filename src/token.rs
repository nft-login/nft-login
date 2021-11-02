use chrono::{Duration, Utc};
use openidconnect::core::{
    CoreGenderClaim, CoreJsonWebKeyType, CoreJweContentEncryptionAlgorithm,
    CoreJwsSigningAlgorithm, CoreRsaPrivateSigningKey, CoreTokenType,
};
use openidconnect::{
    AccessToken, AdditionalClaims, Audience, EmptyExtraTokenFields, IdToken, IdTokenClaims,
    IdTokenFields, IssuerUrl, JsonWebKeyId, StandardClaims, StandardTokenResponse,
    SubjectIdentifier, TokenResponse,
};
use rocket::form::Form;
use rocket::State;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::config::Config;
use crate::web3;

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
    let token = token(config, client_id, nonce, None, None, None).await;
    token.id_token().unwrap().to_string()
}

#[post("/token", data = "<post_data>")]
pub async fn post_token_endpoint(tokens: &State<Tokens>, post_data: Form<PostData>) -> String {
    let mutex = tokens.muted.lock().unwrap();
    let token = mutex.get(&post_data.code);
    match token {
        Some(token) => token.to_string(),
        _ => "".to_string(),
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Claims {
    pub account: String,
    pub nonce: String,
    pub signature: String,
}

impl AdditionalClaims for Claims {}

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
    account: Option<String>,
    signature: Option<String>,
    node_provider: Option<String>,
) -> NftTokenResponse {
    let rsa_pem = config.rsa_pem.clone();

    let claims = match node_provider {
        Some(node_provider) => {
            let is_owner = web3::is_nft_owner_of(
                client_id.clone(),
                account.clone().unwrap_or_default(),
                node_provider,
            )
            .await
            .unwrap();
            if is_owner {
                Claims {
                    account: account.clone().unwrap_or_default(),
                    nonce: nonce.clone().unwrap_or_default(),
                    signature: signature.clone().unwrap_or_default(),
                }
            } else {
                Claims {
                    account: "".to_string(),
                    nonce: nonce.clone().unwrap_or_default(),
                    signature: "".to_string(),
                }
            }
        }
        None => Claims {
            account: account.clone().unwrap_or_default(),
            nonce: nonce.clone().unwrap_or_default(),
            signature: signature.clone().unwrap_or_default(),
        },
    };

    let id_token = IdToken::new(
        IdTokenClaims::new(
            IssuerUrl::new(config.ext_hostname.clone()).unwrap(),
            vec![Audience::new(client_id)],
            Utc::now() + Duration::seconds(300),
            Utc::now(),
            StandardClaims::new(SubjectIdentifier::new(account.clone().unwrap_or_default())),
            claims,
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

    NftTokenResponse::new(
        AccessToken::new(nonce.unwrap_or_default()),
        CoreTokenType::Bearer,
        NftIdTokenFields::new(Some(id_token), EmptyExtraTokenFields {}),
    )
}
