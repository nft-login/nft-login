use rocket::response::content;
use rocket::State;
use serde::Deserialize;
use std::collections::HashMap;

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
    pub key_id: String,
    pub node_provider: HashMap<String, String>,
    pub chain_id: HashMap<String, i32>,
    pub rsa_pem: Option<String>,
}

pub fn get_chain_id(config: &Config, realm: &String) -> i32 {
    // return default kovan
    let numeric = realm.parse::<i32>();
    match numeric {
        Ok(ok) => match config.chain_id.values().any(|&val| val == ok) {
            true => ok,
            false => 42,
        },
        Err(_) => *config.chain_id.get(realm).unwrap_or(&42),
    }
}

pub fn get_node(config: &Config, realm: &String) -> String {
    let chain_id = get_chain_id(config, realm);

    let node = config
        .chain_id
        .iter()
        .find_map(|(key, &val)| if val == chain_id { Some(key) } else { None });

    match node {
        Some(node) => config.node_provider.get(node).unwrap().clone(),
        _ => config
            .node_provider
            .get(&"default".to_string())
            .unwrap()
            .clone(),
    }
}

#[get("/.well-known/openid-configuration")]
pub fn default_configuration(config: &State<Config>) -> content::Json<String> {
    configuration(config, "default".into())
}

#[get("/<realm>/authorize/.well-known/openid-configuration")]
pub fn authorize_well_known(config: &State<Config>, realm: String) -> content::Json<String> {
    configuration(config, realm)
}

#[get("/.well-known/oauth-authorization-server/<realm>/authorize")]
pub fn well_known_oauth_authorization_server(
    config: &State<Config>,
    realm: String,
) -> content::Json<String> {
    configuration(config, realm)
}

#[get("/<realm>/.well-known/openid-configuration")]
pub fn configuration(config: &State<Config>, realm: String) -> content::Json<String> {
    let provider_metadata = CoreProviderMetadata::new(
        IssuerUrl::new(format!("{}/{}", config.ext_hostname, realm)).unwrap(),
        AuthUrl::new(format!("{}/{}/authorize", config.ext_hostname, realm)).unwrap(),
        JsonWebKeySetUrl::new(format!("{}/{}/jwk", config.ext_hostname, realm)).unwrap(),
        vec![
            ResponseTypes::new(vec![CoreResponseType::Code]),
            ResponseTypes::new(vec![CoreResponseType::Token, CoreResponseType::IdToken]),
        ],
        vec![CoreSubjectIdentifierType::Pairwise],
        vec![CoreJwsSigningAlgorithm::RsaSsaPssSha256],
        EmptyAdditionalProviderMetadata {},
    )
    .set_token_endpoint(Some(
        TokenUrl::new(format!("{}/{}/token", config.ext_hostname, realm)).unwrap(),
    ))
    .set_userinfo_endpoint(Some(
        UserInfoUrl::new(format!("{}/{}/userinfo", config.ext_hostname, realm)).unwrap(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use rocket::http::Status;
    use rocket::local::blocking::Client;
    use serde_json::Value;
    use std::collections::HashMap;

    use crate::rocket;

    #[test]
    fn test_chain_id() {
        let config = Config {
            ext_hostname: "".to_string(),
            key_id: "".to_string(),
            node_provider: HashMap::from([("example".into(), "https://example.com".into())]),
            chain_id: HashMap::from([("main".into(), 1)]),
            rsa_pem: None,
        };
        assert_eq!(get_chain_id(&config, &"main".to_string()), 1);
        assert_eq!(get_chain_id(&config, &"unknown".to_string()), 42);
        assert_eq!(get_chain_id(&config, &"1".to_string()), 1);
        assert_eq!(get_chain_id(&config, &"2".to_string()), 42);
    }

    #[test]
    fn test_node() {
        let config = Config {
            ext_hostname: "".to_string(),
            key_id: "".to_string(),
            node_provider: HashMap::from([("default".into(), "https://example.com".into())]),
            chain_id: HashMap::from([("default".into(), 1)]),
            rsa_pem: None,
        };
        assert_eq!(
            get_node(&config, &"example".to_string()),
            "https://example.com"
        );
        assert_eq!(
            get_node(&config, &"unknown".to_string()),
            "https://example.com"
        );
        assert_eq!(get_node(&config, &"1".to_string()), "https://example.com");
    }

    #[test]
    fn test_endpoints() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client.get("/.well-known/openid-configuration").dispatch();
        assert_eq!(response.status(), Status::Ok);
        let response = client
            .get("/kovan/.well-known/openid-configuration")
            .dispatch();
        assert_eq!(response.status(), Status::Ok);
        let response = client
            .get("/.well-known/oauth-authorization-server/kovan/authorize")
            .dispatch();
        assert_eq!(response.status(), Status::Ok);
        let response = client
            .get("/kovan/authorize/.well-known/openid-configuration")
            .dispatch();
        assert_eq!(response.status(), Status::Ok);
        let config = response.into_json::<Value>().unwrap();
        assert_eq!(
            config.get("authorization_endpoint").unwrap(),
            "http://localhost:8000/kovan/authorize"
        );
        assert!(config
            .get("userinfo_endpoint")
            .unwrap()
            .as_str()
            .unwrap()
            .ends_with("kovan/userinfo"));
    }
}
