#[macro_use]
extern crate rocket;

#[macro_use]
extern crate rocket_include_static_resources;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use rocket::State;
use rocket_include_static_resources::{EtagIfNoneMatch, StaticContextManager, StaticResponse};

use openidconnect::core::{CoreJsonWebKeySet, CoreRsaPrivateSigningKey};
use openidconnect::{JsonWebKeyId, PrivateSigningKey};

mod authorize;
mod config;
mod token;
mod web3;

use authorize::authorize_endpoint;
use config::{configuration, Config};
use token::{post_token_endpoint, token_endpoint, Tokens};

cached_static_response_handler! {
    259_200;
    "/index.js" => cached_indexjs => "indexjs",
    "/index.css" => cached_indexcss => "indexcss",
}

#[get("/")]
fn index(
    static_resources: &State<StaticContextManager>,
    etag_if_none_match: EtagIfNoneMatch,
) -> StaticResponse {
    static_resources.build(&etag_if_none_match, "index")
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
    let rocket = rocket::build();
    let figment = rocket.figment();
    let config: Config = figment.extract().expect("config");

    println!("{:?}", config);

    let config = Config {
        ext_hostname: config.ext_hostname.clone(),
        rsa_pem: Some(include_str!("../do-not-use.pem").to_string()),
    };

    let tokens: Tokens = Tokens {
        muted: Arc::new(Mutex::new(HashMap::new())),
    };

    rocket
        .attach(static_resources_initializer!(
            "indexjs" => "static/index.js",
            "indexcss" => "static/index.css",
            "index" => ("static", "index.html"),
        ))
        .mount("/", routes![cached_indexjs, cached_indexcss])
        .mount(
            "/",
            routes![
                index,
                authorize_endpoint,
                token_endpoint,
                post_token_endpoint,
                configuration,
                jwk
            ],
        )
        .manage(config)
        .manage(tokens)
}
