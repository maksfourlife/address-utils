use clap::{Parser, Subcommand};
use ethers_core::{
    k256::{ecdsa::SigningKey, schnorr::CryptoRngCore},
    types::{Address, H256, U256},
    utils::{get_contract_address, get_create2_address_from_hash, secret_key_to_address},
};
use std::{
    fmt::Debug,
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
    /// Number of cores to use
    #[clap(long, default_value_t = num_cpus::get() - 1)]
    pub n_cores: usize,
    /// Debug info every n iterations
    #[clap(long, default_value_t = 100000)]
    pub n_iter: usize,
    #[clap(subcommand)]
    pub command: Command,
}

/// Generates salt that is used in create-2
#[derive(Debug, Clone, Copy, Parser)]
struct Create2Address {
    /// deployer contract
    #[clap(long)]
    pub factory: Address,
    /// deployed contract initialization code hash
    #[clap(long)]
    pub codehash: H256,
}

#[derive(Debug, Clone, Copy, Parser)]
struct ContractAddress {
    /// next deployer nonce
    #[clap(long, default_value_t = U256::zero())]
    pub nonce: U256,
}

#[derive(Debug, Copy, Clone, Subcommand)]
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

fn generate_address(rng: &mut impl CryptoRngCore, target: &Address, mask: &Address) -> bool {
    let private_key = SigningKey::random(rng);
    let address = secret_key_to_address(&private_key);
    if test_address(&address, target, mask) {
        println!("address: {address:?}");
        println!("privateKey: {:?}", private_key.to_bytes());
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
    let private_key = SigningKey::random(rng);
    let address = secret_key_to_address(&private_key);
    let contract = get_contract_address(address, nonce);
    if test_address(&contract, target, mask) {
        println!("contract: {contract:?}");
        println!("address: {address:?}");
        println!("privateKey: {:?}", private_key.to_bytes());
        true
    } else {
        false
    }
}

fn generate_create2_address(
    rng: &mut impl CryptoRngCore,
    target: &Address,
    mask: &Address,
    factory: &Address,
    codehash: &H256,
) -> bool {
    let salt = H256::random_using(rng);
    let contract = get_create2_address_from_hash(*factory, salt, codehash);
    if test_address(&contract, target, mask) {
        println!("contract: {contract:?}");
        println!("salt: {salt:?}");
        true
    } else {
        false
    }
}

fn _generate_contract_address(
    rng: &mut impl CryptoRngCore,
    nonce: &U256,
) -> (Address, Box<dyn Debug>) {
    let private_key = SigningKey::random(rng);
    let address = secret_key_to_address(&private_key);
    let contract = get_contract_address(address, nonce);
    (contract, Box::new((private_key.to_bytes(), address)))
}

fn _generate_address(rng: &mut impl CryptoRngCore) -> (Address, Box<dyn Debug>) {
    let private_key = SigningKey::random(rng);
    let address = secret_key_to_address(&private_key);
    (address, Box::new(private_key.to_bytes()))
}

fn _generate_create2_address(
    rng: &mut impl CryptoRngCore,
    factory: &Address,
    codehash: &H256,
) -> (Address, Box<dyn Debug>) {
    let salt = H256::random_using(rng);
    let contract = get_create2_address_from_hash(*factory, salt, codehash);
    (contract, Box::new(salt))
}

fn main() {
    let Cli {
        command,
        target,
        mask,
        n_cores,
        ..
    } = Cli::parse();

    let exit = Arc::new(AtomicBool::new(false));

    let mut handles = vec![];

    for _ in 0..n_cores {
        let exit = exit.clone();
        handles.push(thread::spawn(move || {
            let mut rng = rand::thread_rng();
            loop {
                if exit.load(Ordering::Relaxed) {
                    return;
                }
                let success = match &command {
                    Command::Address => generate_address(&mut rng, &target, &mask),
                    Command::ContractAddress(ContractAddress { nonce }) => {
                        generate_contract_address(&mut rng, &target, &mask, nonce)
                    }
                    Command::Create2Address(Create2Address { factory, codehash }) => {
                        generate_create2_address(&mut rng, &target, &mask, factory, codehash)
                    }
                };
                if success {
                    exit.store(true, Ordering::Relaxed);
                }
                // let (address, params) = match &command {
                //     Command::Address => _generate_address(&mut rng),
                //     Command::ContractAddress(ContractAddress { nonce }) => {
                //         _generate_contract_address(&mut rng, nonce)
                //     }
                //     Command::Create2Address(Create2Address { factory, codehash }) => {
                //         _generate_create2_address(&mut rng, factory, codehash)
                //     }
                // };
                // if test_address(&address, &target, &mask) {
                //     exit.store(true, Ordering::Relaxed);
                //     println!("{params:?}");
                // }
            }
        }));
    }

    for hdl in handles {
        hdl.join().unwrap();
    }
}
