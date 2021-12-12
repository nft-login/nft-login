#[cfg(test)]
mod client_test {
    use crate::rocket;
    use rocket::http::Status;
    use rocket::local::blocking::Client;

    #[test]
    fn hello_world() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client.get("/").dispatch();
        assert_eq!(response.status(), Status::Ok);
    }
}

#[cfg(test)]
mod jwk_test {
    use crate::rocket;
    use rocket::http::Status;
    use rocket::local::blocking::Client;

    #[test]
    fn hello_world() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client.get("/jwk").dispatch();
        assert_eq!(response.status(), Status::Ok);
        let response = client.get("/kovan/jwk").dispatch();
        assert_eq!(response.status(), Status::Ok);
    }
}

#[cfg(test)]
mod config_test {
    use crate::rocket;
    use rocket::http::Status;
    use rocket::local::blocking::Client;
    use serde_json::Value;

    #[test]
    fn configuration() {
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

#[cfg(test)]
mod authorize_test {
    use crate::rocket;
    use rocket::http::Status;
    use rocket::local::blocking::Client;
    use std::collections::HashMap;
    use url::Url;

    #[test]
    fn redirect() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client.get("/authorize?client_id=0xa0d4E5CdD89330ef9d0d1071247909882f0562eA&realm=kovan&redirect_uri=unused").dispatch();
        assert_eq!(response.status(), Status::TemporaryRedirect);
        let response_url = Url::parse(response.headers().get("Location").next().unwrap()).unwrap();
        let mut path_segments = response_url.path_segments().unwrap();
        assert_eq!(path_segments.next(), Some("kovan"));

        let params: HashMap<String, String> = response_url
            .query()
            .map(|v| {
                url::form_urlencoded::parse(v.as_bytes())
                    .into_owned()
                    .collect()
            })
            .unwrap_or_else(HashMap::new);

        assert_eq!(params.get("realm"), Some(&"kovan".to_string()));

        assert_eq!(params.get("chain_id"), Some(&"kovan".to_string()));

        assert_eq!(
            params.get("contract"),
            Some(&"0xa0d4E5CdD89330ef9d0d1071247909882f0562eA".to_string())
        );
    }
}
