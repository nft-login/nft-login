#[macro_use]
extern crate rocket;
use rocket::State;

use openidconnect::core::{
    CoreClaimName, CoreJsonWebKeySet, CoreJwsSigningAlgorithm, CoreProviderMetadata,
    CoreResponseType, CoreRsaPrivateSigningKey, CoreSubjectIdentifierType,
};
use openidconnect::{
    AuthUrl, EmptyAdditionalProviderMetadata, IssuerUrl, JsonWebKeyId, JsonWebKeySetUrl,
    PrivateSigningKey, ResponseTypes, Scope, TokenUrl,
};

struct Config {
    pub ext_hostname: String,
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/.well-known/openid-configuration")]
fn configuration(config: &State<Config>) -> Result<String, std::io::Error> {
    let provider_metadata = CoreProviderMetadata::new(
        IssuerUrl::new(config.ext_hostname.to_string()).unwrap(),
        AuthUrl::new(format!("{}/authorize", config.ext_hostname).to_string()).unwrap(),
        JsonWebKeySetUrl::new(
            format!("{}/jwk", config.ext_hostname)
                .to_string()
                .to_string(),
        )
        .unwrap(),
        vec![
            ResponseTypes::new(vec![CoreResponseType::Code]),
            ResponseTypes::new(vec![CoreResponseType::Token, CoreResponseType::IdToken]),
        ],
        vec![CoreSubjectIdentifierType::Pairwise],
        vec![CoreJwsSigningAlgorithm::RsaSsaPssSha256],
        EmptyAdditionalProviderMetadata {},
    )
    .set_token_endpoint(Some(
        TokenUrl::new(format!("{}/token", config.ext_hostname).to_string()).unwrap(),
    ))
    .set_scopes_supported(Some(vec![
        Scope::new("openid".to_string()),
        Scope::new("email".to_string()),
        Scope::new("profile".to_string()),
    ]))
    .set_claims_supported(Some(vec![
        CoreClaimName::new("sub".to_string()),
        CoreClaimName::new("aud".to_string()),
        CoreClaimName::new("email".to_string()),
        CoreClaimName::new("email_verified".to_string()),
        CoreClaimName::new("exp".to_string()),
        CoreClaimName::new("iat".to_string()),
        CoreClaimName::new("iss".to_string()),
        CoreClaimName::new("name".to_string()),
        CoreClaimName::new("given_name".to_string()),
        CoreClaimName::new("family_name".to_string()),
        CoreClaimName::new("picture".to_string()),
        CoreClaimName::new("locale".to_string()),
    ]));

    serde_json::to_string(&provider_metadata).map_err(From::from)
}

#[get("/jwk")]
fn jwk() -> String {
    let rsa_pem = include_str!("../do-not-use.pem");
    let jwks = CoreJsonWebKeySet::new(vec![CoreRsaPrivateSigningKey::from_pem(
        &rsa_pem,
        Some(JsonWebKeyId::new("key1".to_string())),
    )
    .expect("Invalid RSA private key")
    .as_verification_key()]);

    serde_json::to_string(&jwks).unwrap()
}

#[launch]
fn rocket() -> _ {
    let config = Config {
        ext_hostname: "https://localhost:8000".to_string(),
    };

    rocket::build()
        .mount("/", routes![index, configuration, jwk])
        .manage(config)
}
