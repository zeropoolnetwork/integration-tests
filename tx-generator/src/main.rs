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
            ff_uint::Num,
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
use serde::Serialize;

pub type PoolParams = PoolBN256;
pub type Fr = <PoolParams as PoolParamsTrait>::Fr;
pub type Fs = <PoolParams as PoolParamsTrait>::Fs;
pub type Engine = Bn256;

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

fn main() {
    let params_bin = std::fs::read("../params/transfer_params.bin").unwrap();
    let params = Parameters::<Bn256>::read(&mut params_bin.as_slice(), true, true).unwrap();

    let rng = &mut rand::thread_rng();
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

    #[derive(Serialize)]
    pub struct ProofWithInputs {
        pub proof: Proof<Engine>,
        pub inputs: Vec<Num<Fr>>,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct TxDataRequest {
        pub tx_type: u16,
        pub proof: ProofWithInputs,
        #[serde(with = "hex")]
        pub memo: Vec<u8>,
        #[serde(with = "hex")]
        pub extra_data: Vec<u8>,
    }

    let (inputs, proof) = tx_proof(&params, tx_data.public.clone(), tx_data.secret.clone());

    let tx_request = TxDataRequest {
        tx_type: 0, // Deposit
        proof: ProofWithInputs { proof, inputs },
        memo: tx_data.memo,
        extra_data: vec![], // Signature for deposit
    };

    let serialized = serde_json::to_string_pretty(&tx_request).unwrap();
    println!("{}", serialized);
}
