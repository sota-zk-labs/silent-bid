use p3_field::PrimeField;
use p3_matrix::dense::RowMajorMatrix;
use crate::{INPUT_ADDRESS_START, INPUT_AMOUNT, NUM_PROVER_COLS, RESULT_ADDRESS_START, RESULT_AMOUNT};
use crate::private_input::PrivateInput;
use crate::public_input::Bid;

pub fn generate_execution_trace<F: PrimeField>(
    public_input: &Vec<Bid>,
    private_input: PrivateInput
) -> RowMajorMatrix<F> {
    let mut values: Vec<F> = Vec::new();
    let mut registers = [F::zero(); NUM_PROVER_COLS];

    for bid in public_input {
        values.extend(registers);
        let d = bid.bidder.as_bytes();

        // add input into register
        for (id, byte) in d.iter().enumerate() {
            registers[id + INPUT_ADDRESS_START] = F::from_canonical_u8(*byte);
        }
        registers[INPUT_AMOUNT] = F::from_canonical_u64(bid.amount);

        if F::from_canonical_u64(bid.amount) > registers[RESULT_AMOUNT] {
            // update result if input amount is higher
            for (id, byte) in d.iter().enumerate() {
                registers[id + RESULT_ADDRESS_START] = F::from_canonical_u8(*byte);
            }
            registers[RESULT_AMOUNT] = F::from_canonical_u64(bid.amount);
        }
    }

    if values.is_empty() {
        values.extend(registers);
        values.extend(registers);
        values.extend(registers);
        values.extend(registers);
    }

    RowMajorMatrix::new(values, NUM_PROVER_COLS)

}