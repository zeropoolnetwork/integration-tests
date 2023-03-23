use std::ptr::null;

use bip32::{
    secp256k1::ecdsa::{
        signature::{hazmat::PrehashSigner, Signer},
        Signature,
    },
    ChildNumber, DerivationPath, ExtendedPrivateKey, Language, Mnemonic, Prefix, XPrv,
};
use bip39::Mnemonic as Bip39Mnemonic;
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
use secp256k1::SecretKey;
use serde::Serialize;
use web3::signing::{keccak256, Key};

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rng = &mut rand::thread_rng();

    let mnemonic = std::env::args().nth(1).unwrap();
    let num_transactions = std::env::args()
        .nth(2)
        .map(|s| s.parse::<u32>().unwrap())
        .unwrap_or(10);

    let m = Bip39Mnemonic::parse_in_normalized(Default::default(), &mnemonic).unwrap();
    let seed = m.to_seed_normalized("");

    let child_path = "m/44'/60'/0'/0".parse::<DerivationPath>()?;

    let params_bin = std::fs::read("../params/transfer_params.bin").unwrap();
    let params = Parameters::<Bn256>::read(&mut params_bin.as_slice(), true, true).unwrap();

    let mut txs = vec![];
    for n in 0..num_transactions {
        let mut path = child_path.clone();
        path.push(ChildNumber::new(n, false)?);
        let child_xprv = XPrv::derive_from_path(&seed, &path)?;

        let sk = rng.gen();
        let tx = generate_transaction(sk, child_xprv, &params);
        txs.push(tx);
    }

    let serialized = serde_json::to_string_pretty(&txs).unwrap();
    println!("{serialized}");
    std::fs::write("../txs.json", serialized).unwrap();

    Ok(())
}

fn generate_transaction(
    sk: Num<Fs>,
    secret_key: XPrv,
    transfer_params: &Parameters<Bn256>,
) -> TxDataRequest {
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

    let (inputs, proof) = tx_proof(
        transfer_params,
        tx_data.public.clone(),
        tx_data.secret.clone(),
    );

    let signing_key = secret_key.private_key();
    let nullifier_bytes = tx_data.public.nullifier.to_uint().0.to_big_endian();
    let signing_key = SecretKey::from_slice(&signing_key.to_bytes()).unwrap();

    fn sign(data: &[u8], key: impl Key) -> Vec<u8> {
        println!("address: {}", key.address());
        let data = [
            b"\x19Ethereum Signed Message:\n",
            data.len().to_string().as_bytes(),
            data,
        ]
        .concat();

        let hash = keccak256(data.as_slice());
        let signature = key.sign(&hash, None).unwrap();

        // println!("recovered: {}", signature.recover(&data).unwrap().address());
        [signature.r.as_bytes(), signature.s.as_bytes()].concat()
    }

    // let signature = Key::sign(&signing_key.into(), &data, None).unwrap();
    let compact = sign(&nullifier_bytes, &signing_key);

    // println!("V: {}", s.v);

    // if s.v == 28 {
    //     compact[]
    // }

    // let signature: Signature = signing_key.sign_prehash(&hash).unwrap();
    // let (r, s) = signature.split_bytes();
    // let sig = RecoverableSignature::from_compact(&r, &s, signature.recovery_id().to_i32()).unwrap();

    // let signature = signature.as_ref().to_vec();

    let tx_request = TxDataRequest {
        tx_type: "0000", // Deposit
        proof: ProofWithInputs { proof, inputs },
        memo: tx_data.memo,
        extra_data: compact, // Signature for deposit
    };

    tx_request
}
