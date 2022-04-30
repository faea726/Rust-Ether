use ethers::{
    prelude::{Middleware, Provider, Signer},
    signers,
};
use eyre::Result;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<()> {
    let prvd = Provider::try_from("https://bscrpc.com")?;

    let private_key = "039d17fedb3da5634bc09a7242c8be5d25f74eb3bdd7287ef8dc9e7e5defc0ec";
    let account = signers::Wallet::from_str(private_key)?;

    let balance_acount = prvd.get_balance(account.address(), None).await?;
    println!("{}", balance_acount);

    Ok(())
}
