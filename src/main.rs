use ethers::{prelude::Provider, signers};
use eyre::Result;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<()> {
    let provider = Provider::try_from("https://bscrpc.com")?;

    let account = signers::Wallet::from_str(
        "039d17fedb3da5634bc09a7242c8be5d25f74eb3bdd7287ef8dc9e7e5defc0ec",
    )?;

    Ok(())
}
