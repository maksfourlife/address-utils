use clap::{Parser, Subcommand};
use ethers_core::{
    k256::{ecdsa::SigningKey, schnorr::CryptoRngCore},
    types::{Address, H256, U256},
    utils::{
        get_contract_address, get_create2_address_from_hash, hex::ToHex, secret_key_to_address,
    },
};
use std::{
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
    thread,
    time::Instant,
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
    pub n_iter: u64,
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

#[derive(Debug)]
enum Output {
    Address(SigningKey),
    ContractAddress(SigningKey, Address),
    Create2Address(H256),
}

impl Display for Output {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            x @ (Self::Address(private_key) | Self::ContractAddress(private_key, ..)) => {
                let private_key = private_key.to_bytes().encode_hex::<String>();
                match x {
                    Self::Address(..) => f
                        .debug_struct("Address")
                        .field("private_key", &private_key)
                        .finish(),
                    Self::ContractAddress(.., deployer) => f
                        .debug_struct("ContractAddress")
                        .field("private_key", &private_key)
                        .field("deployer", deployer)
                        .finish(),
                    _ => unreachable!(),
                }
            }
            Self::Create2Address(salt) => f
                .debug_struct("Create2Address")
                .field("salt", salt)
                .finish(),
        }
    }
}

fn test_address(address: &Address, target: &Address, mask: &Address) -> bool {
    address & mask == target & mask
}

fn generate_address(rng: &mut impl CryptoRngCore) -> (Address, Output) {
    let private_key = SigningKey::random(rng);
    let address = secret_key_to_address(&private_key);
    (address, Output::Address(private_key))
}

fn generate_contract_address(rng: &mut impl CryptoRngCore, nonce: &U256) -> (Address, Output) {
    let private_key = SigningKey::random(rng);
    let address = secret_key_to_address(&private_key);
    let contract = get_contract_address(address, nonce);
    (contract, Output::ContractAddress(private_key, address))
}

fn generate_create2_address(
    rng: &mut impl CryptoRngCore,
    factory: &Address,
    codehash: &H256,
) -> (Address, Output) {
    let salt = H256::random_using(rng);
    let contract = get_create2_address_from_hash(*factory, salt, codehash);
    (contract, Output::Create2Address(salt))
}

fn main() {
    let Cli {
        command,
        target,
        mask,
        n_cores,
        n_iter,
        ..
    } = Cli::parse();

    let exit = Arc::new(AtomicBool::new(false));

    let iter = Arc::new(AtomicU64::new(0));
    let elapsed = Arc::new(AtomicU64::new(0));

    let mut handles = vec![];

    for _ in 0..n_cores {
        let exit = exit.clone();
        let iter = iter.clone();
        let elapsed = elapsed.clone();
        handles.push(thread::spawn(move || {
            let mut rng = rand::thread_rng();
            loop {
                let now = Instant::now();
                if exit.load(Ordering::Relaxed) {
                    return;
                }
                let (address, output) = match &command {
                    Command::Address => generate_address(&mut rng),
                    Command::ContractAddress(ContractAddress { nonce }) => {
                        generate_contract_address(&mut rng, nonce)
                    }
                    Command::Create2Address(Create2Address { factory, codehash }) => {
                        generate_create2_address(&mut rng, factory, codehash)
                    }
                };
                if test_address(&address, &target, &mask) {
                    exit.store(true, Ordering::Relaxed);
                    println!("address = {address:?}");
                    println!("output = {output}");
                }

                let _elapsed =
                    elapsed.fetch_add(now.elapsed().as_nanos() as u64, Ordering::Relaxed);
                let iter = iter.fetch_add(1, Ordering::Relaxed);

                if iter % n_iter == 0 {
                    elapsed.store(0, Ordering::Relaxed);
                    println!("iter = {iter}, avg = {}", _elapsed / n_iter);
                }
            }
        }));
    }

    for hdl in handles {
        hdl.join().unwrap();
    }
}
