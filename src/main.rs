#[macro_use]
extern crate rocket;

use openidconnect::core::{CoreJsonWebKeySet, CoreRsaPrivateSigningKey};
use openidconnect::{JsonWebKeyId, PrivateSigningKey};

mod authorize;
mod config;
mod token;

use authorize::authorize_endpoint;
use config::{configuration, Config};
use token::token_endpoint;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
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
        rsa_pem: include_str!("../do-not-use.pem").to_string(),
    };

    rocket::build()
        .mount(
            "/",
            routes![
                index,
                authorize_endpoint,
                token_endpoint,
                configuration,
                jwk
            ],
        )
        .manage(config)
}
