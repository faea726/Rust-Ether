use ethers::{
    abi::{Abi, Address},
    contract::Contract,
    prelude::{BlockNumber, Middleware, Multicall, Provider, Signer, U256},
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

    // Get ABI data and create Contract
    let contract_address = Address::from_str("0xD99D1c33F9fC3444f8101754aBC46c52416550D1")?;
    let path = "./abis/ERC20-abi.json";
    let file = File::open(path)?;
    let contract_abi: Abi = serde_json::from_reader(file)?;
    let contract = Contract::new(contract_address, contract_abi, &provider);

    // Call
    let call = contract.method::<_, String>("totlSupply", ())?;
    let mut multicall = Multicall::new(&provider, None).await?;
    multicall.add_call(call);
    let result = multicall.call().await?;

    Ok(())
}

fn from_wei(amount: U256, decimals: u8) -> f64 {
    let _amount = amount.as_u128() as f64;
    _amount / 10_f64.powf(decimals as f64)
}

fn to_wei(amount: f64, decimals: u8) -> U256 {
    U256::from_dec_str(&(amount * 10_f64.powf(decimals as f64)).to_string()).unwrap()
}
