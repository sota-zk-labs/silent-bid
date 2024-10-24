use p3_field::{AbstractField, Field, PrimeField};
use p3_goldilocks::Goldilocks;
use p3_matrix::dense::RowMajorMatrix;
use crate::columns::{BidCols, NUM_PROVER_COLS, READ_BYTES};
use crate::private_input::PrivateInput;
use crate::public_input::PublicBid;
use crate::utils::hex_to_bytes;

pub fn generate_execution_trace<F: Field>(
    public_input: &[PublicBid],
    private_input: &PrivateInput<F>,
    d: u32,
    n: u32,
) -> RowMajorMatrix<F> {

    let mut values: Vec<BidCols<F>> = Vec::new();
    let mut registers = BidCols::<F>::default();

    let one = F::from_canonical_u32(1);
    let zero = F::zero();
    let exponent = private_input.private_exponent;
    let modulus = private_input.private_modulus;
    let u16_lim: u64 = u16::MAX as u64 + 1;
    let u32_max: u64 = u32::MAX as u64;

    for public_bid in public_input {
        let mut lim = 1;
        registers = BidCols::<F>::default();
        let mut encrypted_amount = hex_to_bytes(&public_bid.encrypted_amount).unwrap();
        while encrypted_amount.len() % 4 != 0 {
            encrypted_amount.push(0);
        }

        let mut start = 0;
        let end = encrypted_amount.len();
        let mut final_value = 0;
        let mut is_error = 0;
        while start < end {
            let mut _vec: [u8; 4] = encrypted_amount[start..start + 4].try_into().expect("slice with incorrect length");
            let encoded_vec: [F; 4] = _vec.iter().map(|e| F::from_canonical_u8(*e)).collect::<Vec<F>>().try_into().expect("slice with incorrect length");
            let mut decoded_vec = [F::zero(); 4];
            let read_value = F::from_canonical_u32(u32::from_le_bytes(_vec));
            let mut current_value = u32::from_le_bytes(_vec) as u64;

            // init
            if is_error == 1 {
                registers.change(one, zero, encoded_vec.clone(), read_value, zero, exponent, zero, one, zero, decoded_vec.clone(), one, one, registers.final_value);
                values.push(registers.clone());
                start += 4;
                continue;
            }
            registers.change(one, zero, encoded_vec.clone(), read_value, zero, exponent, zero, one, zero, decoded_vec.clone(), zero, one, registers.final_value);
            values.push(registers.clone());

            start += 4;

            if current_value == 0 {
                // lim = lim * u16_lim;
                continue;
            }

            // exponent
            let mut exp = d;
            let mut r: u64 = 1;
            let mut q: u64 = 0;
            let mut q_r: u64 = 0;

            while exp != 1 {
                let new_exp = exp / 2;
                let odd = exp % 2;
                if odd == 1 {
                    q_r = (r * current_value) / (n as u64);
                    r = (r * current_value) % (n as u64);
                }
                q = (current_value * current_value) / (n as u64);
                current_value = (current_value  * current_value) % (n as u64);


                registers.change(zero, one, registers.read_bytes, F::from_canonical_u64(current_value), F::from_canonical_u64(q), F::from_canonical_u32(new_exp),
                                 F::from_canonical_u32(odd), F::from_canonical_u64(r), F::from_canonical_u64(q_r), registers.decoded_bytes,
                                 registers.is_error, registers.lim, registers.final_value);

                exp = new_exp;
                values.push(registers.clone());
            }

            // write the decoded value
            q = (current_value * r) / (n as u64);
            let current_value = (current_value * r) % (n as u64);
            if current_value > u32_max {
                is_error = 1;
                registers.change(zero, zero, registers.read_bytes, F::from_canonical_u64(current_value), F::from_canonical_u64(q), zero, zero,
                                 zero, zero, decoded_vec, one, F::from_canonical_u64(lim),  F::from_canonical_u64(final_value));

            } else {
                let decoded_byte = (current_value as u32).to_le_bytes();
                let decoded_vec: [F; 4] = decoded_byte.iter().map(|e| F::from_canonical_u8(*e)).collect::<Vec<F>>().try_into().expect("slice with incorrect length");

                final_value = (final_value + current_value * lim);
                registers.change(zero, zero, registers.read_bytes, F::from_canonical_u64(current_value), F::from_canonical_u64(q), zero, zero,
                                 zero, zero, decoded_vec, zero, F::from_canonical_u64(lim),  F::from_canonical_u64(final_value));
            }

            values.push(registers.clone());
            // lim = (lim * u16_lim) % n;
            lim = lim * u16_lim;
            // break;
            // let r = read_value
        }

    }

    let height = values.len().next_power_of_two();
    while values.len() < height {
        values.push(registers.clone());
    }

    let mut trace = RowMajorMatrix::new(values.iter().flat_map(|r| r.to_vec()).collect(), NUM_PROVER_COLS);
    let rows: &[BidCols<F>] = &values;
    trace

}