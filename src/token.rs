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
    let access_token = mutex.get(&code);
    if access_token.is_none() {
        return Err(NotFound("Invalid Code".to_string()));
    }
    let access_token = access_token.unwrap();
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
    realm: String,
    client_id: String,
    _nonce: Option<String>,
    standard_claims: StandardClaims<CoreGenderClaim>,
    additional_claims: Claims,
    access_token: AccessToken,
    code: AuthorizationCode,
) -> NftTokenResponse {
    let rsa_pem = config.rsa_pem.clone();
    let id_token = IdToken::new(
        IdTokenClaims::new(
            IssuerUrl::new(format!("{}/{}", config.ext_hostname, realm)).unwrap(),
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
            Some(JsonWebKeyId::new(config.key_id.to_string())),
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

#[cfg(test)]
mod tests {
    use crate::rocket;
    use rocket::http::Status;
    use rocket::local::blocking::Client;
    use serde_json::Value;
    use std::collections::HashMap;
    use url::Url;

    #[test]
    fn token() {
        let client_id = "foo";
        let contract = "0x886B6781CD7dF75d8440Aba84216b2671AEFf9A4";
        let account = "0x9c9e8eabd947658bdb713e0d3ebfe56860abdb8d".to_string();
        let nonce = "dotzxrenodo".to_string();
        let signature = "0x87b709d1e84aab056cf089af31e8d7c891d6f363663ff3eeb4bbb4c4e0602b2e3edf117fe548626b8d83e3b2c530cb55e2baff29ca54dbd495bb45764d9aa44c1c".to_string();

        let client = Client::tracked(rocket()).expect("valid rocket instance");

        let response = client
            .get(format!(
                "/authorize?client_id={}&realm=okt&redirect_uri=https://example.com&nonce={}&contract={}&account={}&signature={}",
                client_id, nonce, contract, account, signature
            ))
            .dispatch();
        assert_eq!(response.status(), Status::TemporaryRedirect);
        let response_url = Url::parse(response.headers().get("Location").next().unwrap()).unwrap();

        let params: HashMap<String, String> = response_url
            .query()
            .map(|v| {
                url::form_urlencoded::parse(v.as_bytes())
                    .into_owned()
                    .collect()
            })
            .unwrap_or_else(HashMap::new);

        assert!(params.get("code").is_some());
        let code = params.get("code").unwrap();
        let response = client.get(format!("/token?code={}", code)).dispatch();
        assert_eq!(response.status(), Status::Ok);
        let response = client.get(format!("/okt/token?code={}", code)).dispatch();
        assert_eq!(response.status(), Status::Ok);
        let token = response.into_json::<Value>().unwrap();
        let access_token = token.get("access_token");
        assert!(access_token.is_some());

        let response = client
            .get(format!("/token?code={}", "invalid".to_string()))
            .dispatch();
        assert_eq!(response.status(), Status::NotFound);
    }
}
