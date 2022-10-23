use ethers::{
    abi::{Abi, Token},
    contract::Contract,
    prelude::{
        k256::ecdsa::SigningKey, Address, BlockNumber, Middleware, Provider, Signer,
        SignerMiddleware, Wallet, U256,
    },
    types::TransactionReceipt,
};
use eyre::Result;
use serde_json;
use std::fs::File;

extern crate ini;
use ini::Ini;

#[tokio::main]
async fn main() -> Result<()> {
    let conf = Ini::load_from_file("conf.ini").unwrap();
    let section = conf.section(Some("test")).unwrap();

    let private_key = section.get("PRIVATE_KEY").unwrap();
    let chain_id = section.get("CHAIN_ID").unwrap().parse::<u64>().unwrap();
    let node = section.get("NODE").unwrap();

    let provider = create_provider(node);

    let wallet: Wallet<SigningKey> = private_key.parse()?;
    let wallet = wallet.with_chain_id(chain_id);

    let client = SignerMiddleware::new(provider, wallet);

    // Query wallet information
    let nonce = client
        .get_transaction_count(client.address(), Some(BlockNumber::Latest.into()))
        .await?;
    let eth_balance_wei = client.get_balance(client.address(), None).await?;
    let eth_balance = from_wei(eth_balance_wei, 18);
    println!(
        "Address: {:#x}\nBalance Wei: {}\nFrom Wei: {}\nNonce: {}",
        client.address(),
        eth_balance_wei,
        eth_balance,
        nonce
    );

    // Call
    let token_contract = create_contract(
        "0xae13d989daC2f0dEbFf460aC112a837C89BAa7cd".parse()?,
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
    let transfer_tx = token_contract
        .method::<_, bool>(
            "transfer",
            (
                Token::Address(client.address()),
                Token::Uint(to_wei(0.001, token_decimals)),
            )
                .to_owned(),
        )?
        .gas(to_wei(0.3, 6))
        .gas_price(to_wei(15.0, 9))
        .legacy();

    let receipt = transfer_tx.send().await?.await?.unwrap(); // Send transaction
    println!("Tx Hash: {:#x}", receipt.transaction_hash);

    // get_tx_infor(client.clone(), receipt).await;

    Ok(())
}

#[allow(dead_code)]
async fn get_tx_infor(
    client: SignerMiddleware<Provider<ethers::prelude::Http>, Wallet<SigningKey>>,
    receipt: TransactionReceipt,
) {
    let tx_infor = client
        .get_transaction_receipt(receipt.transaction_hash)
        .await
        .unwrap();

    println!(
        "\n{}",
        serde_json::to_string_pretty(&tx_infor).expect("error: no data")
    );
}

// Create sign_able contract with provider
fn create_contract(
    contract_address: Address,
    abi_path: &str,
    contract_provider: SignerMiddleware<Provider<ethers::prelude::Http>, Wallet<SigningKey>>,
) -> Contract<SignerMiddleware<Provider<ethers::prelude::Http>, Wallet<SigningKey>>> {
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
