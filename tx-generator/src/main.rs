use std::path::PathBuf;

use bip32::{ChildNumber, DerivationPath, XPrv};
use bip39::Mnemonic as Bip39Mnemonic;
use bpaf::Bpaf;
use libzeropool_rs::{
    client::{state::State, TxType, UserAccount},
    libzeropool::{
        circuit::tx::c_transfer,
        fawkes_crypto::{
            backend::bellman_groth16::{
                engines::Bn256,
                prover::{prove, Proof},
                Parameters,
            },
            ff_uint::{Num, Uint},
        },
        native::{
            boundednum::BoundedNum,
            params::{PoolBN256, PoolParams as PoolParamsTrait},
            tx::{TransferPub, TransferSec},
        },
        POOL_PARAMS,
    },
};
use rand::Rng;
use rayon::prelude::*;
use secp256k1::SecretKey;
use serde::Serialize;
use web3::{
    ethabi::Address,
    signing::{keccak256, Key},
};

pub type PoolParams = PoolBN256;
pub type Fr = <PoolParams as PoolParamsTrait>::Fr;
pub type Fs = <PoolParams as PoolParamsTrait>::Fs;
pub type Engine = Bn256;

#[derive(Serialize)]
pub struct ProofWithInputs {
    pub proof: Proof<Engine>,
    pub inputs: Vec<Num<Fr>>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TxDataRequest {
    pub tx_type: &'static str,
    pub proof: ProofWithInputs,
    #[serde(with = "hex")]
    pub memo: Vec<u8>,
    #[serde(with = "hex")]
    pub extra_data: Vec<u8>,
}

fn tx_proof(
    params: &Parameters<Bn256>,
    public: TransferPub<Fr>,
    secret: TransferSec<Fr>,
) -> (Vec<Num<Fr>>, Proof<Bn256>) {
    let circuit = |public, secret| {
        c_transfer(&public, &secret, &*POOL_PARAMS);
    };

    prove(&params, &public, &secret, circuit)
}

#[derive(Bpaf)]
#[bpaf(options)]
struct Args {
    #[bpaf(short)]
    pub mnemonic: String,

    #[bpaf(short, fallback(10))]
    pub num_transactions: usize,

    #[bpaf(short, fallback("txs.json".parse().unwrap()))]
    pub out_path: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = args().run();

    let m = Bip39Mnemonic::parse_in_normalized(Default::default(), &args.mnemonic)?;
    let seed = m.to_seed_normalized("");
    let child_path = "m/44'/60'/0'/0".parse::<DerivationPath>()?;

    let params_bin = std::fs::read("../params/transfer_params.bin")?;
    let params = Parameters::<Bn256>::read(&mut params_bin.as_slice(), true, true)?;

    println!("Generating transactions and deposit signatures...");
    let txs = (0..args.num_transactions)
        .into_par_iter()
        .map_init(
            || rand::thread_rng(),
            |rng, n| {
                let mut path = child_path.clone();
                path.push(ChildNumber::new(n as u32, false).unwrap());
                let child_xprv = XPrv::derive_from_path(&seed, &path).unwrap();

                let sk = rng.gen();
                let state = State::init_test(POOL_PARAMS.clone());
                let account = UserAccount::new(sk, state, POOL_PARAMS.clone());
                let tx_data = account
                    .create_tx(
                        TxType::Deposit {
                            fee: BoundedNum::new_unchecked(Num::ZERO),
                            deposit_amount: BoundedNum::new_unchecked(Num::ONE),
                            outputs: vec![],
                        },
                        None,
                        None,
                    )
                    .unwrap();

                let nullifier_bytes = tx_data.public.nullifier.to_uint().0.to_big_endian();
                let signature = create_signature(child_xprv, &nullifier_bytes);

                (tx_data, signature)
            },
        )
        .collect::<Vec<_>>();

    println!("Generating transaction proofs...");
    let final_txs = txs
        .into_iter()
        .enumerate()
        .map(|(i, (tx, signature))| {
            let start_time = std::time::Instant::now();
            let (inputs, proof) = tx_proof(&params, tx.public.clone(), tx.secret.clone());

            let tx_req = TxDataRequest {
                tx_type: "0000", // Deposit
                proof: ProofWithInputs { proof, inputs },
                memo: tx.memo,
                extra_data: signature.0, // Signature for deposit
            };

            let elapsed = start_time.elapsed();
            println!(
                "{i} {}: Transaction generation time: {}ms",
                signature.1,
                elapsed.as_millis()
            );

            tx_req
        })
        .collect::<Vec<_>>();

    // TODO: Use a streaming writer?
    println!("Serializing transactions...");
    let serialized = serde_json::to_string(&final_txs)?;
    println!("Writing transactions to {}...", args.out_path.display());
    std::fs::write(args.out_path, serialized)?;

    Ok(())
}

fn create_signature(secret_key: XPrv, nullifier_bytes: &[u8]) -> (Vec<u8>, Address) {
    let signing_key = secret_key.private_key();
    let signing_key = SecretKey::from_slice(&signing_key.to_bytes()).unwrap();

    fn sign(data: &[u8], key: impl Key) -> (Vec<u8>, Address) {
        let data = [
            b"\x19Ethereum Signed Message:\n",
            data.len().to_string().as_bytes(),
            data,
        ]
        .concat();

        let hash = keccak256(data.as_slice());
        let signature = key.sign(&hash, None).unwrap();

        (
            [signature.r.as_bytes(), signature.s.as_bytes()].concat(),
            key.address(),
        )
    }

    sign(&nullifier_bytes, &signing_key)
}
