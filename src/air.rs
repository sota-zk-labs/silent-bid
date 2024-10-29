use p3_air::{Air, AirBuilder, AirBuilderWithPublicValues, BaseAir};
use p3_field::{Field, AbstractField};
use p3_matrix::Matrix;
use core::borrow::{Borrow};
use crate::columns::{BidCols, ADDRESS_BYTES, BASE, DECODED_BYTES, NUM_BID_COLS, READ_BYTES};
use crate::public_input::PublicBid;

pub struct  ProverAir{
    pub(crate) public_input: Vec<PublicBid>,
}

impl ProverAir {
    pub fn new(public_input: Vec<PublicBid>) -> Self{
        Self {
            public_input
        }
    }
}

impl <F: Field> BaseAir<F> for ProverAir {
    fn width(&self) -> usize {
        NUM_BID_COLS
    }
}

impl <AB: AirBuilderWithPublicValues> Air<AB> for ProverAir {
    fn eval(&self, builder: &mut AB){
        eval_decryption(builder);
        eval_hashing(builder);
        eval_logic(builder);

    }
}

pub fn eval_decryption<AB: AirBuilderWithPublicValues> (builder: &mut AB) {
    // columns involves: flags, read_bytes, current_value, quotient_value, exponent_value
    // odd_exponent, r, q_r, decoded_bytes, gap, final_value,
    let main = builder.main();

    let modules = builder.public_values()[0].into();
    let local = main.row_slice(0);
    let next = main.row_slice(1);
    let local: &BidCols<AB::Var> = (*local).borrow();
    let next: &BidCols< AB::Var> = (*next).borrow();

    // first row:
    builder.when_first_row().assert_eq(local.new_bidder, AB::F::one());
    builder.when_first_row().assert_eq(local.is_dummy, AB::F::zero());

    // conditions
    let new_bidder = local.new_bidder;
    let is_reading = local.is_reading;


    // new bidder constraints
    builder.when(new_bidder).assert_one(local.r);
    builder.when(new_bidder).assert_one(local.gap);
    builder.when(new_bidder).assert_zero(local.is_reading);
    builder.when(new_bidder).assert_zero(local.is_exponent);
    builder.when(new_bidder).assert_zero(local.current_value);
    builder.when(new_bidder).assert_zero(local.quotient_value);
    builder.when(new_bidder).assert_zero(local.odd_exponent);
    builder.when(new_bidder).assert_zero(local.q_r);
    builder.when(new_bidder).assert_zero(local.is_error);
    builder.when(new_bidder).assert_zero(local.final_value);
    builder.when(new_bidder).assert_one(local.gap);
    for i in 0..READ_BYTES {
        builder.when(new_bidder).assert_zero(local.read_bytes[i]);
    }
    for i in 0..DECODED_BYTES {
        builder.when(new_bidder).assert_zero(local.decoded_bytes[i]);
    }


    // when reading
    let lim1 = AB::F::from_canonical_u64(256);  //2^8
    let lim2 = AB::F::from_canonical_u64(65536);  //2^16
    let lim3 = AB::F::from_canonical_u64(16777216);  //2^8
    let first_read = next.is_reading * local.new_bidder;
    builder.when(is_reading).assert_eq(local.current_value,
                                       local.read_bytes[0] + local.read_bytes[1] * lim1 + local.read_bytes[2] * lim2 + local.read_bytes[3] * lim3);
    builder.when(is_reading).assert_one(local.r);
    builder.when(is_reading).assert_zero(local.quotient_value);
    builder.when(is_reading).assert_zero(local.odd_exponent);
    builder.when(is_reading).assert_zero(local.q_r);
    builder.when(is_reading).assert_zero(local.quotient_value);
    builder.when(first_read).assert_one(next.gap);
    for i in 0..DECODED_BYTES {
        builder.when(is_reading).assert_zero(local.decoded_bytes[i]);
    }
    builder.when(next.is_reading).assert_eq(local.final_value, next.final_value);

    // when exponent

    let next_exponent = next.is_exponent;
    let next_odd_exponent = next.odd_exponent;
    let two = AB::F::from_canonical_u64(2);

    // check exponent
    builder.when(next_exponent).assert_eq(local.exponent_value, next.exponent_value * two + next.odd_exponent);
    // check current value
    builder.when(next_exponent).assert_eq(local.current_value * local.current_value, next.quotient_value * modules.clone() + next.current_value);
    // check reminder
    builder.when(next_odd_exponent).assert_eq(local.r * local.current_value, next.q_r * modules.clone() + next.r);
    // other cells stay the same
    builder.when(next_exponent).assert_eq(local.gap, next.gap);
    for i in 0..DECODED_BYTES {
        builder.when(next_exponent).assert_eq(local.decoded_bytes[i], next.decoded_bytes[i]);
    }
    builder.when(next_exponent).assert_eq(local.final_value, next.final_value);


    // exponent value constraints
    let not_exponent = AB::Expr::one() - local.is_exponent;
    let not_new_bidder = AB::Expr::one() - local.new_bidder;
    let not_reading = AB::Expr::one() - local.is_reading;
    let not_error = AB::Expr::one() - local.is_error;
    builder.when(not_exponent.clone()).when(not_new_bidder.clone()).when(not_reading.clone()).when(not_error.clone()).assert_eq(local.exponent_value, AB::Expr::zero());
    builder.when(not_exponent.clone()).when(not_new_bidder.clone()).when(not_reading.clone()).when(not_error.clone())
        .assert_eq(local.current_value, local.decoded_bytes[0] + local.decoded_bytes[1] * lim1 + local.decoded_bytes[2] * lim2 + local.decoded_bytes[3] * lim3);

    // final value constraints
    let next_not_exponent = AB::Expr::one() - next.is_exponent;
    let next_not_new_bidder = AB::Expr::one() - next.new_bidder;
    let next_not_reading = AB::Expr::one() - next.is_reading;
    let next_not_error = AB::Expr::one() - next.is_error;

    builder.when(next_not_exponent).when(next_not_new_bidder).when(next_not_reading).when(next_not_error)
        .assert_eq(next.final_value, local.final_value + next.current_value * local.gap);

    // check gap constraints
    let gap_diff = AB::F::from_canonical_u64(65536);
    let gap_condition = next.is_reading - next.is_reading * local.new_bidder - next.is_reading * local.is_reading - next.is_reading * next.new_bidder;
    let normal = next.is_exponent;
    builder.when(gap_condition).assert_eq(next.gap, local.gap * gap_diff);
    builder.when(normal).assert_eq(next.gap, local.gap);

    // check error constraints

    let error_before = next.is_error * next.is_reading;
    builder.when(error_before).assert_one(local.is_error);

    builder.when_ne(next.decoded_bytes[2], AB::Expr::zero()).assert_one(next.is_error * (AB::Expr::one() - local.is_error));
    builder.when_ne(next.decoded_bytes[3], AB::Expr::zero()).assert_one(next.is_error * (AB::Expr::one() - local.is_error));
    // cells that must stay the same when is error
    let next_error = next.is_error;
    builder.when(next_error).assert_eq(local.final_value, next.final_value);

}

pub fn eval_hashing<AB: AirBuilderWithPublicValues> (builder: &mut AB) {

    // columns involves: flags, read_bytes, read_address, hash_lim, hash_value

    let main = builder.main();

    let local = main.row_slice(0);
    let next = main.row_slice(1);
    let local: &BidCols<AB::Var> = (*local).borrow();
    let next: &BidCols< AB::Var> = (*next).borrow();
    let base = AB::Expr::from_canonical_u64(BASE as u64);

    // new bidder
    let new_bidder = local.new_bidder;
    let next_new_bidder = next.new_bidder;
    let next_not_dummy = AB::Expr::one() - next.is_dummy;

    builder.when_transition().when(next_new_bidder).when(next_not_dummy.clone()).assert_eq(local.hash_lim, next.hash_lim);

    // check lim constraints
    builder.when(new_bidder).assert_eq(local.hash_lim * base.exp_u64(5), next.hash_lim);
    let read_encrypted_byte = local.is_reading;
    let local_not_dummy = AB::Expr::one() - local.is_dummy;
    builder.when(next_not_dummy.clone()).when(local_not_dummy.clone()).when(read_encrypted_byte).assert_eq(local.hash_lim * base.clone(), next.hash_lim);
    
    // check hash values

    // first hash value
    let mut first_lim = AB::Expr::one();
    let mut first_hash = AB::Expr::zero();
    for i in 0..5 {
        let hash_num = next.read_address[i*4] +  next.read_address[i*4 + 1] * AB::Expr::from_canonical_u64(256)
            + local.read_address[i*4 + 2] * AB::Expr::from_canonical_u64(65536) + next.read_address[i*4 + 3] * AB::Expr::from_canonical_u64(16777216);
        first_hash = first_hash +  hash_num * first_lim.clone();
        first_lim = base.clone() * first_lim;
    }
    builder.when_first_row().assert_eq(local.hash_value, first_hash);

    // check hash during the exponent
    let next_exponent = next.is_exponent;
    let local_exponent = local.is_exponent;
    builder.when(next_exponent).assert_eq(local.hash_value, next.hash_value);
    builder.when(local_exponent).assert_eq(local.hash_lim, next.hash_lim);

    // hash new address value
    let next_new_bidder = next.new_bidder;
    let mut start_lim = next.hash_lim.into();
    let mut new_hash = local.hash_value.into();
    for i in 0..5 {
        let hash_num = next.read_address[i*4] +  next.read_address[i*4 + 1] * AB::Expr::from_canonical_u64(256)
            + local.read_address[i*4 + 2] * AB::Expr::from_canonical_u64(65536) + next.read_address[i*4 + 3] * AB::Expr::from_canonical_u64(16777216);
        new_hash = new_hash +  hash_num * start_lim.clone();
        start_lim = base.clone() * start_lim;
    }
    builder.when(local_not_dummy.clone()).when(next_new_bidder).assert_eq(new_hash, next.hash_value);

    // when reading
    let next_new_reader = next.is_reading;
    builder.when(next_not_dummy.clone()).when(local_not_dummy.clone()).when(next_new_reader).assert_eq(local.hash_value + next.hash_lim * next.current_value, next.hash_value);

    // when error
    let next_error = next.is_error;
    for i in 0..ADDRESS_BYTES {
        builder.when(next_error).assert_eq(local.read_address[i], next.read_address[i]);
    }
    builder.when(next_error).assert_eq(local.hash_lim, next.hash_lim);
    builder.when(next_error).assert_eq(local.hash_value, next.hash_value);

    // check final hash
    let final_hash = builder.public_values()[1];
    builder.when_last_row().assert_eq(local.hash_value, final_hash);
}

pub fn eval_logic<AB: AirBuilderWithPublicValues> (builder: &mut AB) {
    // columns involves: flags, final_value, bid_amount, nonce, winner_amount,
    // change_winner, winner_address
    let main = builder.main();

    let local = main.row_slice(0);
    let next = main.row_slice(1);
    let local: &BidCols<AB::Var> = (*local).borrow();
    let next: &BidCols< AB::Var> = (*next).borrow();

    // first row
    builder.when_first_row().assert_zero(local.winner_amount);
    builder.when_first_row().assert_zero(local.nonce);
    builder.when_first_row().assert_zero(local.bid_amount);
    for i in 0..ADDRESS_BYTES {
        builder.when_first_row().assert_zero(local.winner_address[i]);
    }

    // new bidder
    let new_bidder = local.new_bidder;
    builder.when(new_bidder).assert_zero(local.bid_amount);
    builder.when(new_bidder).assert_zero(local.nonce);
    builder.when(new_bidder).assert_zero(local.change_winner);

    let next_computing_winner = next.computing_winner;
    // check nonce
    builder.when(next_computing_winner.clone()).assert_eq(next.final_value, next.bid_amount * AB::Expr::from_canonical_u64(1000) + next.nonce);


    // check winner
    let next_not_change = AB::Expr::one() - next.change_winner;
    let next_change = next.change_winner;
    let next_is_error = next.is_error;

    // is reading
    let is_reading = local.is_reading;
    let next_reading = next.is_reading;
    builder.when(is_reading).assert_zero(local.bid_amount);
    builder.when(is_reading).assert_zero(local.nonce);
    builder.when(is_reading).assert_zero(local.change_winner);
    builder.when(next_reading).assert_eq(local.winner_amount, next.winner_amount);
    for i in 0..ADDRESS_BYTES{
        builder.when(next_reading).assert_eq(local.winner_address[i], next.winner_address[i]);
    }


    // when exponent
    let next_exponent = next.is_exponent;
    builder.when(next_exponent).assert_eq(local.bid_amount, next.bid_amount);
    builder.when(next_exponent).assert_eq(local.nonce, next.nonce);
    builder.when(next_exponent).assert_eq(local.change_winner, next.change_winner);
    builder.when(next_exponent).assert_eq(local.winner_amount, next.winner_amount);
    for i in 0..ADDRESS_BYTES{
        builder.when(next_exponent).assert_eq(local.winner_address[i], next.winner_address[i]);
    }

    // error
    builder.when(next_computing_winner.clone()).when(next_is_error).assert_zero(next.change_winner);

    // if change
    // new bid amount equals to the previous winner
    builder.when(next_computing_winner.clone()).when(next_change).assert_eq(next.bid_amount, next.winner_amount);
    for i in 0..ADDRESS_BYTES {
        builder.when(next_computing_winner.clone()).when(next_change).assert_eq(next.winner_address[i], next.read_address[i]);
    }

    // if not change
    builder.when(next_computing_winner.clone()).when(next_not_change.clone()).assert_eq(local.winner_amount, next.winner_amount);
    for i in 0..ADDRESS_BYTES {
        builder.when(next_computing_winner.clone()).when(next_not_change.clone()).assert_eq(next.winner_address[i], local.winner_address[i]);
    }
}