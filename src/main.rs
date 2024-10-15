mod air;
mod public_input;
mod private_input;
mod generate_execution_trace;

use std::marker::PhantomData;
use p3_challenger::{ HashChallenger, SerializingChallenger32};
use p3_circle::CirclePcs;
use p3_commit::ExtensionMmcs;
use p3_field::AbstractField;
use p3_field::extension::BinomialExtensionField;
use p3_fri::{FriConfig};
use p3_keccak::Keccak256Hash;
use p3_merkle_tree::MerkleTreeMmcs;
use p3_mersenne_31::{Mersenne31};
use p3_symmetric::{CompressionFunctionFromHasher,  SerializingHasher32};
use p3_uni_stark::{prove, verify, StarkConfig};
use tracing_forest::util::LevelFilter;
use tracing_forest::ForestLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Registry};
use crate::air::ProverAir;
use crate::generate_execution_trace::generate_execution_trace;
use crate::private_input::PrivateInput;

const NUM_PROVER_COLS: usize = 42;
const RESULT_ADDRESS_START: usize = 1;
const RESULT_ADDRESS_END: usize = 20;
const RESULT_AMOUNT: usize = 0;
const INPUT_AMOUNT: usize = 21;
const INPUT_ADDRESS_START: usize = 22;
const INPUT_ADDRESS_END: usize = 41;

fn main() {

    // set log level
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    Registry::default()
        .with(env_filter)
        .with(ForestLayer::default())
        .init();

    // config type
    type Val = Mersenne31;
    type Challenge = BinomialExtensionField<Val, 3>;

    type ByteHash = Keccak256Hash;
    type FieldHash = SerializingHasher32<ByteHash>;
    let byte_hash = ByteHash {};
    let field_hash = FieldHash::new(Keccak256Hash {});

    type MyCompress = CompressionFunctionFromHasher<ByteHash, 2, 32>;
    let compress = MyCompress::new(byte_hash);

    type ValMmcs = MerkleTreeMmcs<Val, u8, FieldHash, MyCompress, 32>;

    let val_mmcs = ValMmcs::new(field_hash, compress);

    type ChallengeMmcs = ExtensionMmcs<Val, Challenge, ValMmcs>;
    let challenge_mmcs = ChallengeMmcs::new(val_mmcs.clone());

    type Challenger = SerializingChallenger32<Val, HashChallenger<u8, ByteHash, 32>>;

    let fri_config = FriConfig {
        log_blowup: 1,
        num_queries: 100,
        proof_of_work_bits: 16,
        mmcs: challenge_mmcs,
    };

    type Pcs = CirclePcs<Val, ValMmcs, ChallengeMmcs>;
    let pcs = Pcs {
        mmcs: val_mmcs,
        fri_config,
        _phantom: PhantomData,
    };

    type MyConfig = StarkConfig<Pcs, Challenge, Challenger>;
    let config = MyConfig::new(pcs);
    let private_input = PrivateInput::new("vjp".to_string());
    let trace = generate_execution_trace::<Val>(&vec![], private_input.clone());

    let mut challenger = Challenger::from_hasher(vec![], byte_hash);



    let pis = vec![
        Mersenne31::from_canonical_u64(0),
        Mersenne31::from_canonical_u64(1),
        Mersenne31::from_canonical_u64(21),
    ];

    let air = ProverAir::new(private_input.clone(), vec![]);
    let proof = prove(&config, &air , &mut challenger, trace, &pis);
    let mut challenger = Challenger::from_hasher(vec![], byte_hash);
    verify(&config, &air, &mut challenger, &proof, &pis).expect("verification failed");

}
