use clap::Parser;
use ethers_core::{
    k256::ecdsa::SigningKey,
    types::{Address, U256},
    utils::get_contract_address,
};
use ethers_signers::{Signer, Wallet};

#[derive(Debug, Parser)]
struct Cli {
    #[clap(long)]
    pub target: Address,
    #[clap(long)]
    pub mask: Address,
}

fn main() {
    let cli = Cli::parse();
    let mut rng = rand::thread_rng();
    loop {
        let signing_key = SigningKey::random(&mut rng);
        let wallet: Wallet<_> = signing_key.clone().into();
        let address = get_contract_address(wallet.address(), U256::zero());
        if address & cli.mask == cli.target & cli.mask {
            dbg!(signing_key);
            dbg!(address);
            break;
        }
    }
}
