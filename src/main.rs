use clap::{Parser, Subcommand};
use ethers_core::{
    k256::{ecdsa::SigningKey, schnorr::CryptoRngCore},
    types::{Address, H256, U256},
    utils::{get_contract_address, get_create2_address_from_hash, hex::ToHex},
};
use ethers_signers::{Signer, Wallet};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
};

#[derive(Debug, Parser)]
struct Cli {
    #[clap(long)]
    pub target: Address,
    #[clap(long)]
    pub mask: Address,
    #[clap(subcommand)]
    pub command: Command,
}

/// Generates salt that is used in create-2
#[derive(Debug, Parser)]
struct Create2Address {
    /// deployer contract
    #[clap(long)]
    pub factory: Address,
    /// deployed contract initialization code hash
    #[clap(long)]
    pub codehash: H256,
}

#[derive(Debug, Parser)]
struct ContractAddress {
    /// next deployer nonce
    #[clap(long)]
    pub nonce: U256,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Generate EOA account address
    Address,
    /// Generate address for contract deployed with nonce
    ContractAddress(ContractAddress),
    /// Generate salt for create-2 deployed contract
    Create2Address(Create2Address),
}

fn test_address(address: &Address, target: &Address, mask: &Address) -> bool {
    address & mask == target & mask
}

#[allow(unused)]
fn generate_address(rng: &mut impl CryptoRngCore, target: &Address, mask: &Address) -> bool {
    let signing_key = SigningKey::random(rng);
    let wallet = Wallet::from(signing_key.clone());
    let address = wallet.address();
    if test_address(&address, target, mask) {
        println!("address: {address:?}");
        println!(
            "privateKey: {:?}",
            signing_key.to_bytes().encode_hex::<String>()
        );
        true
    } else {
        false
    }
}

fn generate_contract_address(
    rng: &mut impl CryptoRngCore,
    target: &Address,
    mask: &Address,
    nonce: &U256,
) -> bool {
    let signing_key = SigningKey::random(rng);
    let wallet = Wallet::from(signing_key.clone());
    let address = wallet.address();
    let contract = get_contract_address(address, nonce);
    if test_address(&contract, target, mask) {
        println!("contract: {contract:?}");
        println!("address: {address:?}");
        println!(
            "privateKey: {:?}",
            signing_key.to_bytes().encode_hex::<String>()
        );
        true
    } else {
        false
    }
}

// todo: optimize
#[allow(unused)]
fn generate_create2_address(
    rng: &mut impl CryptoRngCore,
    target: &Address,
    mask: &Address,
    factory: Address,
    codehash: &H256,
) -> bool {
    let salt = H256::random_using(rng);
    let contract = get_create2_address_from_hash(factory, salt, codehash);
    if test_address(&contract, target, mask) {
        println!("contract: {contract:?}");
        println!("salt: {salt:?}");
        true
    } else {
        false
    }
}

fn main() {
    let cli = Cli::parse();
    let exit = Arc::new(AtomicBool::new(false));
    let num_cpus = num_cpus::get();
    let mut handles = vec![];
    for _ in 0..num_cpus - 1 {
        let exit = exit.clone();
        handles.push(thread::spawn(move || {
            let mut rng = rand::thread_rng();
            loop {
                if exit.load(Ordering::Relaxed) {
                    return;
                }
                if generate_contract_address(&mut rng, &cli.target, &cli.mask, &U256::zero()) {
                    exit.store(true, Ordering::Relaxed);
                }
            }
        }));
    }
    for hdl in handles {
        hdl.join().unwrap();
    }
}
