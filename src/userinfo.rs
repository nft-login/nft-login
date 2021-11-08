use crate::claims::{Claims, ClaimsMutex};
use openidconnect::{core::CoreGenderClaim, UserInfoClaims};
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::response::status::NotFound;
use rocket::serde::json::Json;
use rocket::State;

#[derive(Debug)]
pub struct Bearer(String);

#[derive(Debug)]
pub enum BearerError {
    Missing,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Bearer {
    type Error = BearerError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match req.headers().get_one("Authorization") {
            None => Outcome::Failure((Status::BadRequest, BearerError::Missing)),
            Some(token) => Outcome::Success(Bearer(token.to_string().replace("Bearer ", ""))),
        }
    }
}

#[get("/userinfo")]
pub async fn default_userinfo_endpoint(
    claims: &State<ClaimsMutex>,
    bearer: Bearer,
) -> Result<Json<UserInfoClaims<Claims, CoreGenderClaim>>, NotFound<String>> {
    userinfo_endpoint(claims, bearer, "default".into()).await
}

#[allow(unused_variables)]
#[get("/<realm>/userinfo")]
pub async fn userinfo_endpoint(
    claims: &State<ClaimsMutex>,
    bearer: Bearer,
    realm: String,
) -> Result<Json<UserInfoClaims<Claims, CoreGenderClaim>>, NotFound<String>> {
    println!("{:?}", bearer);

    let access_token = bearer.0.clone();

    let locked = claims.standard_claims.lock().unwrap();
    let standard_claims = locked.get(&access_token).unwrap();

    let locked = claims.additional_claims.lock().unwrap();
    let additional_claims = locked.get(&access_token).unwrap();

    let userinfo_claims = UserInfoClaims::new(standard_claims.clone(), additional_claims.clone());

    Ok(Json(userinfo_claims))
}

#[options("/userinfo")]
pub async fn default_options_userinfo_endpoint() {}

#[allow(unused_variables)]
#[options("/<realm>/userinfo")]
pub async fn options_userinfo_endpoint(realm: String) {}
