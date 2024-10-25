use p3_air::{Air, AirBuilder, AirBuilderWithPublicValues, BaseAir};
use p3_field::{Field, AbstractField};
use p3_matrix::Matrix;
use core::borrow::{Borrow, BorrowMut};
use crate::columns::{BidCols, DECODED_BYTES, NUM_BID_COLS, READ_BYTES};
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
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();

        let modulos = builder.public_values()[0].into();
        let local = main.row_slice(0);
        let next = main.row_slice(1);
        let local: &BidCols<AB::Var> = (*local).borrow();
        let next: &BidCols< AB::Var> = (*next).borrow();

        // first row:
        builder.when_first_row().assert_eq(local.new_bidder, AB::F::one());

        // conditions
        let new_bidder = local.new_bidder;
        let is_reading = local.is_reading;
        let is_exponent = local.is_exponent;
        let is_error = local.is_error;


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
        for i in 0..DECODED_BYTES {
            builder.when(new_bidder).assert_zero(local.decoded_bytes[i]);
        }
        builder.when(first_read).assert_one(next.gap);

        // when exponent

        let next_exponent = next.is_exponent;
        let next_r = next.r;

        let next_odd_exponent = next.odd_exponent;

        let two = AB::F::from_canonical_u64(2);

        // check exponent
        builder.when(next_exponent).assert_eq(local.exponent_value, next.exponent_value * two + next.odd_exponent);
        // check current value
        builder.when(next_exponent).assert_eq(local.current_value * local.current_value, next.quotient_value * modulos.clone() + next.current_value);
        // check reminder
        builder.when(next_odd_exponent).assert_eq(local.r * local.current_value, next.q_r * modulos.clone() + next.r);

        // check value
        let not_exponent = AB::Expr::one() - local.is_exponent;
        let not_new_bidder = AB::Expr::one() - local.new_bidder;
        let not_reading = AB::Expr::one() - local.is_reading;
        let not_error = AB::Expr::one() - local.is_error;
        builder.when(not_exponent.clone()).when(not_new_bidder.clone()).when(not_reading.clone()).when(not_error.clone()).assert_eq(local.exponent_value, AB::Expr::zero());
        builder.when(not_exponent.clone()).when(not_new_bidder.clone()).when(not_reading.clone()).when(not_error.clone())
            .assert_eq(local.current_value, local.decoded_bytes[0] + local.decoded_bytes[1] * lim1 + local.decoded_bytes[2] * lim2 + local.decoded_bytes[3] * lim3);

        // final value
        let next_not_exponent = AB::Expr::one() - next.is_exponent;
        let next_not_new_bidder = AB::Expr::one() - next.new_bidder;
        let next_not_reading = AB::Expr::one() - next.is_reading;
        let next_not_error = AB::Expr::one() - next.is_error;

        builder.when(next_not_exponent).when(next_not_new_bidder).when(next_not_reading).when(next_not_error)
            .assert_eq(next.final_value, local.final_value + next.current_value * local.gap);

        // check gap
        let gap_diff = AB::F::from_canonical_u64(65536);
        let gap_condition = next.is_reading - next.is_reading * local.new_bidder - next.is_reading * local.is_reading - next.is_reading * next.new_bidder;
        let normal = next.is_exponent;
        builder.when(gap_condition).assert_eq(next.gap, local.gap * gap_diff);
        builder.when(normal).assert_eq(next.gap, local.gap);

        // check error
        let error_before = next.is_error * next.is_reading;
        builder.when(error_before).assert_one(local.is_error);

        builder.when_ne(next.decoded_bytes[2], AB::Expr::zero()).assert_one(next.is_error * (AB::Expr::one() - local.is_error));
        builder.when_ne(next.decoded_bytes[3], AB::Expr::zero()).assert_one(next.is_error * (AB::Expr::one() - local.is_error));

    }
}
