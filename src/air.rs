use p3_air::{Air, AirBuilder, AirBuilderWithPublicValues, BaseAir};
use p3_field::{Field, AbstractField};
use p3_matrix::Matrix;
use crate::columns::{BidCols, NUM_PROVER_COLS};
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
        NUM_PROVER_COLS
    }
}

impl <AB: AirBuilderWithPublicValues> Air<AB> for ProverAir {
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();

        let local = main.row_slice(0);
        let next = main.row_slice(1);
        builder.when_first_row().assert_eq(local[0], AB::F::one());
        let local: &BidCols<AB::Var> = (*local).borrow();
        let next: &BidCols< AB::Var> = (*next).borrow();

    }
}
