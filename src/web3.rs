use std::str::FromStr;
use web3::signing::{keccak256, recover};

use web3::{
    contract::{Contract, Options},
    types::{Address, U256},
};

pub fn validate(account: String, nonce: String, signature: String) -> bool {
    let message = eth_message(format!("{}{}", account, nonce));
    let signature = format!("{}", &signature[2..]);
    let signature = hex::decode(signature).unwrap();
    let pubkey0 = recover(&message, &signature[..64], 0);
    let pubkey1 = recover(&message, &signature[..64], 1);
    let pubkey0 = format!("{:02X?}", pubkey0.unwrap());
    let pubkey1 = format!("{:02X?}", pubkey1.unwrap());
    pubkey0 == account || pubkey1 == account
}

pub async fn is_nft_owner_of(
    contract_address: String,
    owner_address: String,
    node_provider: String,
) -> web3::Result<bool> {
    let transport = web3::transports::Http::new(&node_provider).unwrap();
    let web3 = web3::Web3::new(transport);
    let abi = include_bytes!("erc721.json");
    let contract_address = Address::from_str(&contract_address).unwrap();
    let contract = Contract::from_json(web3.eth(), contract_address, abi).unwrap();

    let owner_address = Address::from_str(&owner_address).unwrap();
    let balance: U256 = contract
        .query(
            "balanceOf",
            (owner_address,),
            None,
            Options::default(),
            None,
        )
        .await
        .unwrap();
    Ok(balance > U256::from(0))
}

pub fn eth_message(message: String) -> [u8; 32] {
    keccak256(
        format!(
            "{}{}{}",
            "\x19Ethereum Signed Message:\n",
            message.len(),
            message
        )
        .as_bytes(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use web3::signing::{keccak256, recover};

    pub fn eth_message(message: String) -> [u8; 32] {
        keccak256(
            format!(
                "{}{}{}",
                "\x19Ethereum Signed Message:\n",
                message.len(),
                message
            )
            .as_bytes(),
        )
    }

    #[test]
    fn test_recover() {
        let account = "0x63f9a92d8d61b48a9fff8d58080425a3012d05c8".to_string();
        let message = "0x63f9a92d8d61b48a9fff8d58080425a3012d05c8igwyk4r1o7o".to_string();
        let message = eth_message(message);
        let signature = hex::decode("382a3e04daf88f322730f6a2972475fc5646ea8c4a7f3b5e83a90b10ba08a7364cd2f55348f2b6d210fbed7fc485abf19ecb2f3967e410d6349dd7dd1d4487751b").unwrap();
        println!("{} {:?} {:?}", account, message, signature);
        let pubkey = recover(&message, &signature[..64], 0);
        assert!(pubkey.is_ok());
        let pubkey = pubkey.unwrap();
        let pubkey = format!("{:02X?}", pubkey);
        assert_eq!(account, pubkey)
    }

    #[test]
    fn test_validate() {
        let account = "0x63f9a92d8d61b48a9fff8d58080425a3012d05c8".to_string();
        let nonce = "igwyk4r1o7o".to_string();
        let signature = "0x382a3e04daf88f322730f6a2972475fc5646ea8c4a7f3b5e83a90b10ba08a7364cd2f55348f2b6d210fbed7fc485abf19ecb2f3967e410d6349dd7dd1d4487751b".to_string();

        assert!(validate(account, nonce, signature));
    }

    #[tokio::test]
    async fn test_crypto_kitties() {
        let rocket = rocket::build();
        let figment = rocket.figment();
        let config: crate::config::Config = figment.extract().expect("config");

        let ck_token_addr = "0x06012c8cf97BEaD5deAe237070F9587f8E7A266d".to_string();
        let owner = "0xb1690c08e213a35ed9bab7b318de14420fb57d8c".to_string();
        assert!(is_nft_owner_of(ck_token_addr, owner, config.node_provider)
            .await
            .unwrap());
    }
}
