

use rocket::response::Redirect;




use crate::token::token;

pub struct Config {
    pub ext_hostname: String,
}

#[get("/authorize")]
pub async fn authorize_endpoint() -> Redirect {
    let token = token().await;
    println!("{}", token);
    Redirect::temporary(format!("http://localhost:8000/?token={}", token))
}
