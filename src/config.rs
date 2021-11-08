use rocket::response::content;
use rocket::State;
use serde::Deserialize;

use openidconnect::core::{
    CoreClaimName, CoreJwsSigningAlgorithm, CoreProviderMetadata, CoreResponseType,
    CoreSubjectIdentifierType,
};
use openidconnect::{
    AuthUrl, EmptyAdditionalProviderMetadata, IssuerUrl, JsonWebKeySetUrl, ResponseTypes, Scope,
    TokenUrl, UserInfoUrl,
};

#[derive(Debug, PartialEq, Deserialize)]
pub struct Config {
    pub ext_hostname: String,
    pub node_provider: String,
    pub rsa_pem: Option<String>,
}

#[get("/.well-known/openid-configuration")]
pub fn configuration(config: &State<Config>) -> content::Json<String> {
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
    .set_userinfo_endpoint(Some(
        UserInfoUrl::new(format!("{}/userinfo", config.ext_hostname).to_string()).unwrap(),
    ))
    .set_scopes_supported(Some(vec![
        Scope::new("openid".to_string()),
        Scope::new("email".to_string()),
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
    ]));

    content::Json(serde_json::to_string(&provider_metadata).unwrap())
}
