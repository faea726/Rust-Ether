use ethers::{
    abi::{Abi, Token},
    contract::Contract,
    prelude::{
        k256::ecdsa::SigningKey, Address, BlockNumber, Middleware, Provider, Signer,
        SignerMiddleware, Wallet, U256,
    },
};
use eyre::Result;
use serde_json;
use std::{fs::File, str::FromStr};

static NODE: &str = "https://bscrpc.com"; // Main net: ChainID: 56_u64
static CHAIN_ID: u64 = 56;
// static NODE: &str = "https://data-seed-prebsc-1-s1.binance.org:8545/"; // Test net: ChainID: 97_u64
// static CHAIN_ID: u64 = 97;

static PRIVATE_KEY: &str = "039d17fedb3da5634bc09a7242c8be5d25f74eb3bdd7287ef8dc9e7e5defc0ec";

#[tokio::main]
async fn main() -> Result<()> {
    let provider = create_provider(NODE);

    let wallet: Wallet<SigningKey> = PRIVATE_KEY.parse()?;
    let wallet = wallet.with_chain_id(CHAIN_ID);

    let client = SignerMiddleware::new(provider, wallet);

    // Query wallet information
    let nonce = client
        .get_transaction_count(client.address(), Some(BlockNumber::Latest.into()))
        .await?;
    let eth_balance_wei = client.get_balance(client.address(), None).await?;
    let eth_balance = from_wei(eth_balance_wei, 18);
    println!(
        "Address: {}\nBalance Wei: {}\nFrom Wei: {}\nNonce: {}",
        client.address(),
        eth_balance_wei,
        eth_balance,
        nonce
    );

    // Call
    let token_contract = create_contract(
        "0x8076c74c5e3f5852037f31ff0093eeb8c8add8d3",
        "./abis/ERC20-abi.json",
        client.clone(),
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

    // Transfer
    let transfer = token_contract
        .method::<_, bool>(
            "transfer",
            (
                Token::Address(token_contract.address()),
                Token::Uint(to_wei(100_f64, token_decimals)),
            )
                .to_owned(),
        )?
        .gas(to_wei(300000.0, 0))
        .gas_price(to_wei(15.0, 9))
        .legacy();

    let receipt = transfer.send().await?.await?.unwrap();
    println!("Tx Hash: {}", receipt.transaction_hash);

    Ok(())
}

// Create sign_able contract with provider
fn create_contract(
    contract_address: &str,
    abi_path: &str,
    contract_provider: SignerMiddleware<Provider<ethers::prelude::Http>, Wallet<SigningKey>>,
) -> Contract<SignerMiddleware<Provider<ethers::prelude::Http>, Wallet<SigningKey>>> {
    let contract_address = Address::from_str(contract_address).expect("Not Address");

    let file = File::open(abi_path).expect("No JSON file");
    let contract_abi: Abi = serde_json::from_reader(file).expect("Wrong JSON format");

    Contract::new(contract_address, contract_abi, contract_provider)
}

fn create_provider(node: &str) -> Provider<ethers::prelude::Http> {
    Provider::try_from(node).expect("Wrong node")
}

fn from_wei(amount: U256, decimals: u8) -> f64 {
    let _amount = amount.as_u128() as f64;
    _amount / 10_f64.powf(decimals as f64)
}

fn to_wei(amount: f64, decimals: u8) -> U256 {
    U256::from_dec_str(&(amount * 10_f64.powf(decimals as f64)).to_string()).unwrap()
}
