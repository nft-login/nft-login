use openidconnect::core::CoreGenderClaim;
use openidconnect::{
    AdditionalClaims, EndUserEmail, EndUserName, StandardClaims, SubjectIdentifier,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub account: String,
    pub nonce: String,
    pub signature: String,
    pub chain_id: i32,
    pub node: String,
    pub contract: String,
}

impl AdditionalClaims for Claims {}

pub struct ClaimsMutex {
    pub standard_claims: Arc<Mutex<HashMap<String, StandardClaims<CoreGenderClaim>>>>,
    pub additional_claims: Arc<Mutex<HashMap<String, Claims>>>,
}

pub fn standard_claims(account: &String) -> StandardClaims<CoreGenderClaim> {
    StandardClaims::new(SubjectIdentifier::new(account.clone()))
        .set_email(Some(EndUserEmail::new("no-reply@example.com".to_string())))
        .set_email_verified(Some(false))
        .set_name(Some(EndUserName::new("anonymous".to_string()).into()))
}

pub fn additional_claims(
    account: &String,
    nonce: &String,
    signature: &String,
    chain_id: &i32,
    node: &String,
    contract: &String,
) -> Claims {
    Claims {
        account: account.clone(),
        nonce: nonce.clone(),
        signature: signature.clone(),
        chain_id: *chain_id,
        node: node.clone(),
        contract: contract.clone(),
    }
}
