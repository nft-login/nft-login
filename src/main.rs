#[macro_use]
extern crate rocket;

#[macro_use]
extern crate rocket_include_static_resources;

use claims::ClaimsMutex;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::State;
use rocket::{Request, Response};
use rocket_include_static_resources::{EtagIfNoneMatch, StaticContextManager, StaticResponse};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use openidconnect::core::{CoreJsonWebKeySet, CoreRsaPrivateSigningKey};
use openidconnect::{JsonWebKeyId, PrivateSigningKey};

mod authorize;
mod claims;
mod config;
mod tests;
mod token;
mod userinfo;
mod web3;

use authorize::{authorize_endpoint, default_authorize_endpoint};
use config::{
    authorize_well_known, configuration, default_configuration,
    well_known_oauth_authorization_server, Config,
};
use token::{
    default_post_token_endpoint, default_token_endpoint, post_token_endpoint, token_endpoint,
    Tokens,
};
use userinfo::{
    default_options_userinfo_endpoint, default_userinfo_endpoint, options_userinfo_endpoint,
    userinfo_endpoint,
};

cached_static_response_handler! {
    259_200;
    "/index.js" => cached_indexjs => "indexjs",
    "/index.css" => cached_indexcss => "indexcss",
}

#[get("/")]
fn default_index(
    static_resources: &State<StaticContextManager>,
    etag_if_none_match: EtagIfNoneMatch,
) -> StaticResponse {
    static_resources.build(&etag_if_none_match, "index")
}

#[allow(unused_variables)]
#[get("/<realm>")]
fn index(
    static_resources: &State<StaticContextManager>,
    etag_if_none_match: EtagIfNoneMatch,
    realm: String,
) -> StaticResponse {
    static_resources.build(&etag_if_none_match, "index")
}

#[get("/jwk")]
fn default_jwk(config: &State<Config>) -> String {
    jwk(config, "".into())
}

#[get("/<_realm>/jwk")]
fn jwk(config: &State<Config>, _realm: String) -> String {
    let rsa_pem = include_str!("../do-not-use.pem");
    let jwks = CoreJsonWebKeySet::new(vec![CoreRsaPrivateSigningKey::from_pem(
        rsa_pem,
        Some(JsonWebKeyId::new(config.key_id.to_string())),
    )
    .expect("Invalid RSA private key")
    .as_verification_key()]);

    serde_json::to_string(&jwks).unwrap()
}

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Attaching CORS headers to responses",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, GET, PATCH, OPTIONS",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

#[catch(401)]
fn unauthorized() -> String {
    "We could not find a token for your address on this contract.".to_string()
}

#[launch]
pub fn rocket() -> _ {
    let rocket = rocket::build();
    let figment = rocket.figment();
    let mut config: Config = figment.extract().expect("config");

    println!("{:?}", config);

    config.rsa_pem = Some(include_str!("../do-not-use.pem").to_string());

    let tokens: Tokens = Tokens {
        muted: Arc::new(Mutex::new(HashMap::new())),
        bearer: Arc::new(Mutex::new(HashMap::new())),
    };

    let claims: ClaimsMutex = ClaimsMutex {
        standard_claims: Arc::new(Mutex::new(HashMap::new())),
        additional_claims: Arc::new(Mutex::new(HashMap::new())),
    };

    rocket
        .attach(static_resources_initializer!(
            "indexjs" => "static/index.js",
            "indexcss" => "static/index.css",
            "index" => ("static", "index.html"),
        ))
        .attach(CORS)
        .mount("/", routes![cached_indexjs, cached_indexcss])
        .mount(
            "/",
            routes![
                index,
                default_index,
                authorize_endpoint,
                default_authorize_endpoint,
                token_endpoint,
                default_token_endpoint,
                userinfo_endpoint,
                default_userinfo_endpoint,
                options_userinfo_endpoint,
                default_options_userinfo_endpoint,
                post_token_endpoint,
                default_post_token_endpoint,
                configuration,
                default_configuration,
                authorize_well_known,
                well_known_oauth_authorization_server,
                jwk,
                default_jwk
            ],
        )
        .manage(config)
        .manage(tokens)
        .manage(claims)
        .register("/", catchers![unauthorized])
}
