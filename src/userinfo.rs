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

    let access_token = bearer.0;

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

#[cfg(test)]
mod tests {
    use crate::rocket;
    use rocket::http::{Header, Status};
    use rocket::local::blocking::Client;
    use serde_json::Value;
    use std::collections::HashMap;
    use url::Url;

    #[test]
    fn userinfo() {
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
        println!("{:?}", token);
        let access_token = token.get("access_token");
        assert!(access_token.is_some());
        let access_token = access_token.unwrap().as_str().unwrap().to_string();

        let response = client
            .get(format!("/token?code={}", "invalid".to_string()))
            .dispatch();
        assert_eq!(response.status(), Status::NotFound);

        let response = client.get("/userinfo").dispatch();
        assert_eq!(response.status(), Status::BadRequest);

        let response = client
            .get("/userinfo")
            .header(Header::new(
                "Authorization",
                format!("Bearer {}", access_token),
            ))
            .dispatch();
        assert_ne!(response.status(), Status::BadRequest);
        let userinfo = response.into_json::<Value>().unwrap();
        
        assert_eq!(userinfo.get("account").unwrap().as_str().unwrap(), account);
        assert_eq!(userinfo.get("contract").unwrap().as_str().unwrap(), contract);
        assert_eq!(userinfo.get("nonce").unwrap().as_str().unwrap(), nonce);
    }
}
