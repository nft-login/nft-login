#[macro_use]
extern crate rocket;

#[macro_use]
extern crate rocket_include_static_resources;

use rocket::State;
use rocket_include_static_resources::{EtagIfNoneMatch, StaticContextManager, StaticResponse};

use openidconnect::core::{CoreJsonWebKeySet, CoreRsaPrivateSigningKey};
use openidconnect::{JsonWebKeyId, PrivateSigningKey};

mod authorize;
mod config;
mod token;

use authorize::authorize_endpoint;
use config::{configuration, Config};
use token::token_endpoint;

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
    let config = Config {
        ext_hostname: "http://localhost:8000".to_string(),
        rsa_pem: include_str!("../do-not-use.pem").to_string(),
    };

    rocket::build()
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
                configuration,
                jwk
            ],
        )
        .manage(config)
}
