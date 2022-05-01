use ethers::{
    abi::{Abi, Address},
    contract::Contract,
    prelude::{BlockNumber, Middleware, Provider, Signer, U256},
    signers,
};
use eyre::Result;
use serde_json;
use std::{fs::File, str::FromStr};

static NODE: &str = "https://bscrpc.com"; // Main net
static PRIVATE_KEY: &str = "039d17fedb3da5634bc09a7242c8be5d25f74eb3bdd7287ef8dc9e7e5defc0ec";
// static NODE: &str = "https://data-seed-prebsc-1-s1.binance.org:8545/"; // Test net

#[tokio::main]
async fn main() -> Result<()> {
    // Connect to Blockchain
    let provider = Provider::try_from(NODE)?;

    // Create account based on private key
    let account = signers::Wallet::from_str(PRIVATE_KEY)?;
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
    let token_contract = create_contract(
        "0x8076c74c5e3f5852037f31ff0093eeb8c8add8d3",
        "./abis/ERC20-abi.json",
    );

    let token_decimals = token_contract
        .method::<_, u8>("decimals", ())?
        .call()
        .await?;
    let token_symbol = token_contract
        .method::<_, String>("symbol", ())?
        .call()
        .await?;
    let total_supply = token_contract
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

fn create_contract(
    contract_address: &str,
    abi_path: &str,
) -> Contract<Provider<ethers::prelude::Http>> {
    let contract_provider = Provider::try_from(NODE).expect("Wrong Node");

    let contract_address = Address::from_str(contract_address).expect("Not Address");
    let abi = contract_abi(abi_path);

    Contract::new(contract_address, abi, contract_provider)
}

fn contract_abi(path: &str) -> Abi {
    let file = File::open(path).expect("No JSON file");
    serde_json::from_reader(file).expect("Wrong JSON format")
}

fn from_wei(amount: U256, decimals: u8) -> f64 {
    let _amount = amount.as_u128() as f64;
    _amount / 10_f64.powf(decimals as f64)
}

fn to_wei(amount: f64, decimals: u8) -> U256 {
    U256::from_dec_str(&(amount * 10_f64.powf(decimals as f64)).to_string()).unwrap()
}
