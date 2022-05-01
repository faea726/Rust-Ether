use ethers::{
    abi::{Abi, Address},
    contract::Contract,
    prelude::{BlockNumber, Middleware, Provider, Signer, U256},
    signers,
};
use eyre::Result;
use serde_json;
use std::{fs::File, str::FromStr};

#[allow(unused_variables)]
#[tokio::main]
async fn main() -> Result<()> {
    // Connect to Blockchain
    let provider = Provider::try_from("https://bscrpc.com")?;
    // let provider = Provider::try_from("https://data-seed-prebsc-1-s1.binance.org:8545/")?;

    // Create account based on private key
    let private_key = "039d17fedb3da5634bc09a7242c8be5d25f74eb3bdd7287ef8dc9e7e5defc0ec";
    let account = signers::Wallet::from_str(private_key)?;
    let nonce = provider
        .get_transaction_count(account.address(), Some(BlockNumber::Latest.into()))
        .await?;

    // Query account information
    let eth_balance_wei = provider.get_balance(account.address(), None).await?;
    let eth_balance = from_wei(eth_balance_wei, 18);
    println!(
        "Address: {}\nBalance Wei: {}\nFrom Wei: {}\nTo Wei: {}\nNonce: {}",
        account.address(),
        eth_balance_wei,
        eth_balance,
        to_wei(eth_balance, 18),
        nonce
    );

    // Call
    let token_decimals = token_contract()
        .method::<_, u8>("decimals", ())?
        .call()
        .await?;
    let token_symbol = token_contract()
        .method::<_, String>("symbol", ())?
        .call()
        .await?;
    let total_supply = token_contract()
        .method::<_, U256>("totalSupply", ())?
        .call()
        .await?;

    println!(
        "{}({}): {}",
        token_symbol,
        token_decimals,
        from_wei(total_supply, token_decimals)
    );

    Ok(())
}

fn token_contract() -> Contract<Provider<ethers::prelude::Http>> {
    let contract_provider = Provider::try_from("https://bscrpc.com").expect("Wrong Node");
    let contract_address =
        Address::from_str("0x8076c74c5e3f5852037f31ff0093eeb8c8add8d3").expect("Not Address");
    let file = File::open("./abis/ERC20-abi.json").expect("msg");
    let contract_abi: Abi = serde_json::from_reader(file).expect("No JSON file");

    Contract::new(contract_address, contract_abi, contract_provider)
}

fn from_wei(amount: U256, decimals: u8) -> f64 {
    let _amount = amount.as_u128() as f64;
    _amount / 10_f64.powf(decimals as f64)
}

fn to_wei(amount: f64, decimals: u8) -> U256 {
    U256::from_dec_str(&(amount * 10_f64.powf(decimals as f64)).to_string()).unwrap()
}
