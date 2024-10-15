use p3_air::{Air, AirBuilder, AirBuilderWithPublicValues, BaseAir};
use p3_field::{Field, AbstractField};
use p3_matrix::Matrix;
use crate::{NUM_PROVER_COLS};
use crate::private_input::PrivateInput;
use crate::public_input::Bid;

pub struct ProverAir {
    private_input: PrivateInput,
    public_input: Vec<Bid>,
}

impl ProverAir {
    pub fn new(private_input: PrivateInput, public_input: Vec<Bid>) -> Self{
        Self {
            private_input,
            public_input
        }
    }
}

impl <F: Field> BaseAir<F> for ProverAir {
    fn width(&self) -> usize {
        NUM_PROVER_COLS
    }
}

impl <AB: AirBuilderWithPublicValues> Air<AB> for ProverAir {
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local = main.row_slice(0);
        let next = main.row_slice(1);

        // Enforce starting values
        for id in 0..NUM_PROVER_COLS {
            builder.when_first_row().assert_eq(local[id], AB::Expr::zero());
        }

        // Enforce state transition constraints

        // if the input amount is higher than current amount
        // if builder.when_transition(). (next[INPUT_AMOUNT]local[RESULT_AMOUNT].into() {
        //     // the result amount and address must be updated
        //     builder.when_transition().assert_eq(next[INPUT_AMOUNT], next[RESULT_AMOUNT]);
        //     for i in INPUT_ADDRESS_START..(INPUT_ADDRESS_END+1) {
        //         builder.when_transition().assert_eq(next[i], next[i - INPUT_ADDRESS_START + RESULT_ADDRESS_START]);
        //     }
        // } else {
        //     // the result amount and address must stay the same
        //     builder.when_transition().assert_eq(next[RESULT_AMOUNT], local[RESULT_AMOUNT]);
        //     for i in RESULT_ADDRESS_START..(RESULT_ADDRESS_END+1) {
        //         builder.when_transition().assert_eq(next[i], local[i]);
        //     }
        // }
    }
}
