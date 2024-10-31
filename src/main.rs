#![feature(step_trait)]

mod air;
mod public_input;
mod private_input;
mod generate_execution_trace;
mod utils;
mod columns;

use p3_challenger::{HashChallenger, SerializingChallenger64};
use p3_commit::ExtensionMmcs;
use p3_dft::Radix2DitParallel;
use p3_field::AbstractField;
use p3_field::extension::BinomialExtensionField;
use p3_fri::{FriConfig, TwoAdicFriPcs};
use p3_keccak::Keccak256Hash;
use p3_merkle_tree::MerkleTreeMmcs;
use p3_goldilocks::Goldilocks;
use p3_symmetric::{CompressionFunctionFromHasher, SerializingHasher64};
use p3_uni_stark::{prove, verify, StarkConfig};
use tracing_forest::util::LevelFilter;
use tracing_forest::ForestLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Registry};
use crate::air::ProverAir;
use crate::generate_execution_trace::generate_execution_trace;
use crate::private_input::PrivateInput;
use crate::public_input::PublicBid;
use crate::utils::address_to_bytes;

fn main() {
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    Registry::default()
        .with(env_filter)
        .with(ForestLayer::default())
        .init();

    type Val = Goldilocks;
    type Challenge = BinomialExtensionField<Val, 2>;

    type ByteHash = Keccak256Hash;
    type FieldHash = SerializingHasher64<ByteHash>;
    let byte_hash = ByteHash {};
    let field_hash = FieldHash::new(byte_hash);

    type MyCompress = CompressionFunctionFromHasher<ByteHash, 2, 32>;
    let compress = MyCompress::new(byte_hash);

    type ValMmcs = MerkleTreeMmcs<Val, u8, FieldHash, MyCompress, 32>;
    let val_mmcs = ValMmcs::new(field_hash, compress);

    type ChallengeMmcs = ExtensionMmcs<Val, Challenge, ValMmcs>;
    let challenge_mmcs = ChallengeMmcs::new(val_mmcs.clone());

    type Dft = Radix2DitParallel<Val>;
    let dft = Dft::default();

    type Challenger = SerializingChallenger64<Val, HashChallenger<u8, ByteHash, 32>>;


    let fri_config = FriConfig {
        log_blowup: 3,
        num_queries: 80,
        proof_of_work_bits: 16,
        mmcs: challenge_mmcs,
    };
    type Pcs = TwoAdicFriPcs<Val, Dft, ValMmcs, ChallengeMmcs>;
    let pcs = Pcs::new(dft, val_mmcs, fri_config);

    type MyConfig = StarkConfig<Pcs, Challenge, Challenger>;
    let config = MyConfig::new(pcs);


    let private_input = PrivateInput::new(Goldilocks::from_canonical_u64(1875143437), Goldilocks::from_canonical_u64(561461413));
    let bidders = vec![
        PublicBid {bidder: "0x95222290DD7278Aa3Ddd389Cc1E1d165CC4BAfe5".to_string(), encrypted_amount: "211be84e0b6176170000000000000000".to_string()},
        PublicBid {bidder: "0x95222290DD7278Aa3Ddd389Cc1E1d165CC4BAfdd".to_string(), encrypted_amount: "d1188229623701140000000000000000".to_string()},
        PublicBid {bidder: "0x95222290DD7278Aa3Ddd389Cc1E1d165CC4BAfdd".to_string(), encrypted_amount: "8f8eb7400b6176170000000000000000".to_string()},
    ];

    let (trace, winner_add, winner_amount, hash_value) = generate_execution_trace(&bidders, &private_input, 561461413, 1875143437);
    println!("hash input: {:?}", hash_value);
    println!("winner: {:?}, amount: {:?}", winner_add, winner_amount);
    let address_bytes = address_to_bytes(&winner_add);
    let address_bytes_goldilocks: [Goldilocks; 20] = address_bytes.iter().map(|e| Goldilocks::from_canonical_u8(*e)).collect::<Vec<Goldilocks>>().try_into().expect("slice with incorrect length");

    let mut public_input = vec![
        Goldilocks::from_canonical_u64(1875143437), //modulo
        hash_value, // hash value
        Goldilocks::from_canonical_u64(311), // base in Rolling Hash
        winner_amount,
    ];
    public_input.extend(address_bytes_goldilocks);

    let mut challenger = Challenger::from_hasher(vec![], byte_hash);

    let air = ProverAir {public_input: bidders};
    let proof = prove(&config, &air , &mut challenger, trace, &public_input);
    let mut challenger = Challenger::from_hasher(vec![], byte_hash);
    verify(&config, &air, &mut challenger, &proof, &public_input).expect("verification failed");

}
