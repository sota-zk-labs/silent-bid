use p3_field::{AbstractField, Field};
use p3_matrix::dense::RowMajorMatrix;
use crate::columns::{BidCols, BASE, DECODED_BYTES, NUM_BID_COLS, READ_BYTES};
use crate::private_input::PrivateInput;
use crate::public_input::PublicBid;
use crate::utils::{address_to_bytes, find_diff_pos, hex_to_bytes, u64_to_bits};

pub fn generate_execution_trace<F: Field>(
    public_input: &[PublicBid],
    private_input: &PrivateInput<F>,
    d: u32,
    n: u32,
) -> RowMajorMatrix<F> {

    let mut values: Vec<BidCols<F>> = Vec::new();
    let mut registers = BidCols::<F>::default();
    registers.hash_lim = F::one();
    // values.push(registers.clone());

    let one = F::from_canonical_u32(1);
    let zero = F::zero();
    let exponent = private_input.private_exponent;
    let modulus = private_input.private_modulus;
    let u16_gap: u64 = u16::MAX as u64 + 1;
    let u16_max: u64 = u16::MAX as u64;
    let base = F::from_canonical_u64(311);
    let mut winner_amount = 0;
    let mut change_winner = 0;
    for public_bid in public_input {
        change_winner = 0;
        let mut encrypted_amount = hex_to_bytes(&public_bid.encrypted_amount).unwrap();
        while encrypted_amount.len() % 4 != 0 {
            encrypted_amount.push(0);
        }

        let mut address_bytes: [u8; 20] = address_to_bytes(&public_bid.bidder)[0..20].try_into().expect("wrong address");

        new_bidder(&mut registers, exponent, &address_bytes);
        let address: [F; 20] = address_bytes.iter().map(|e| F::from_canonical_u8(*e)).collect::<Vec<F>>().try_into().expect("slice with incorrect length");
        values.push(registers.clone());
        registers.hash_lim = registers.hash_lim * base.exp_u64(5);
        let mut start = 0;
        let end = encrypted_amount.len();
        let mut final_value = 0;
        let mut is_error = 0;
        let mut gap = 1;
        while start < end {
            let mut _vec: [u8; 4] = encrypted_amount[start..start + 4].try_into().expect("slice with incorrect length");
            let encoded_vec: [F; 4] = _vec.iter().map(|e| F::from_canonical_u8(*e)).collect::<Vec<F>>().try_into().expect("slice with incorrect length");
            let mut decoded_vec = [F::zero(); 4];
            let read_value = F::from_canonical_u32(u32::from_le_bytes(_vec));
            registers.hash_value = registers.hash_value + read_value * registers.hash_lim;
            let mut current_value = u32::from_le_bytes(_vec) as u64;


            // init
            if is_error == 1 {
                registers.change(zero, zero, one, zero, zero, encoded_vec.clone(), read_value, zero, exponent, zero, one, zero, decoded_vec.clone(),
                                 one, F::from_canonical_u64(gap), registers.final_value, registers.read_address, registers.hash_lim, registers.hash_value,
                                 registers.bid_amount, registers.nonce, registers.winner_amount, registers.change_winner, registers.winner_address);
                values.push(registers.clone());
                start += 4;
                registers.hash_lim = registers.hash_lim * base;
                continue;
            } else if current_value == 0 {
                registers.change(zero, zero, one, zero, zero, encoded_vec.clone(), read_value, zero, zero, zero, one, zero, decoded_vec.clone(), zero,
                                 F::from_canonical_u64(gap), registers.final_value, registers.read_address, registers.hash_lim, registers.hash_value,
                                 registers.bid_amount, registers.nonce, registers.winner_amount,  registers.change_winner, registers.winner_address);
                values.push(registers.clone());
                start += 4;
                registers.hash_lim = registers.hash_lim * base;
                continue;
            }

            registers.change(zero, zero, one, zero, zero, encoded_vec.clone(), read_value, zero, exponent, zero, one, zero, decoded_vec.clone(), zero,
                             F::from_canonical_u64(gap), registers.final_value, registers.read_address, registers.hash_lim, registers.hash_value,
                             registers.bid_amount, registers.nonce, registers.winner_amount,
                             registers.change_winner, registers.winner_address);
            values.push(registers.clone());

            start += 4;
            registers.hash_lim = registers.hash_lim * base;


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

                registers.change(zero, zero, zero, one, zero, registers.read_bytes, F::from_canonical_u64(current_value), F::from_canonical_u64(q), F::from_canonical_u32(new_exp),
                                 F::from_canonical_u32(odd), F::from_canonical_u64(r), F::from_canonical_u64(q_r), registers.decoded_bytes,
                                 registers.is_error, registers.gap, registers.final_value, registers.read_address, registers.hash_lim, registers.hash_value,
                                 registers.bid_amount, registers.nonce,  registers.winner_amount, registers.change_winner, registers.winner_address);

                exp = new_exp;
                values.push(registers.clone());
            }

            // write the decoded value
            q = (current_value * r) / (n as u64);
            let current_value = (current_value * r) % (n as u64);
            if current_value > u16_max {
                is_error = 1;
                let decoded_byte = (current_value as u32).to_le_bytes();
                let decoded_vec: [F; 4] = decoded_byte.iter().map(|e| F::from_canonical_u8(*e)).collect::<Vec<F>>().try_into().expect("slice with incorrect length");
                registers.change(zero, zero, zero, zero, zero, registers.read_bytes, F::from_canonical_u64(current_value), F::from_canonical_u64(q), zero, zero,
                                 zero, zero, decoded_vec, one, registers.gap, F::from_canonical_u64(final_value), registers.read_address, registers.hash_lim, registers.hash_value,
                                 registers.bid_amount, registers.nonce, registers.winner_amount, registers.change_winner, registers.winner_address);

            } else {
                let decoded_byte = (current_value as u32).to_le_bytes();
                let decoded_vec: [F; 4] = decoded_byte.iter().map(|e| F::from_canonical_u8(*e)).collect::<Vec<F>>().try_into().expect("slice with incorrect length");

                final_value = (final_value + current_value * gap);
                registers.change(zero, zero, zero, zero, zero, registers.read_bytes, F::from_canonical_u64(current_value), F::from_canonical_u64(q), zero, zero, one, zero,
                                 decoded_vec, zero, registers.gap, F::from_canonical_u64(final_value), registers.read_address, registers.hash_lim, registers.hash_value,
                                 registers.bid_amount, registers.nonce, registers.winner_amount, registers.change_winner, registers.winner_address);
            }

            values.push(registers.clone());
            // gap = (lim * u16_lim) % n;
            gap = gap * u16_gap;
            // break;
            // let r = read_value
        }


        // compute answer
        if is_error == 1 {
            registers.change_winner = zero;
            registers.change(zero, zero, zero, zero, one, registers.read_bytes, registers.current_value, registers.quotient_value, registers.exponent_value,
                             registers.odd_exponent, registers.r, registers.q_r, registers.decoded_bytes, registers.is_error, registers.gap, registers.final_value,
                             registers.read_address, registers.hash_lim, registers.hash_value, registers.bid_amount, registers.nonce,
                             registers.winner_amount, zero, registers.winner_address);
        } else {
            let nonce = final_value % 1000;
            let bid_amount = final_value / 1000;
            // let pos = find_diff_pos(bid_amount, winner_amount);
            if bid_amount > winner_amount {
                winner_amount = bid_amount;
                registers.winner_amount = F::from_canonical_u64(winner_amount);
                registers.winner_address = address;
                change_winner = 1;

            } else {
                change_winner = 0;
            }

            // let _bits = u64_to_bits(bid_amount);
            // let bid_amount_bits: [F; 64] = _bits.iter().map(|e| F::from_canonical_u8(*e)).collect::<Vec<F>>().try_into().expect("slice with incorrect length");
            // let _bits = u64_to_bits(winner_amount);
            // let winner_amount_bits: [F; 64] = _bits.iter().map(|e| F::from_canonical_u8(*e)).collect::<Vec<F>>().try_into().expect("slice with incorrect length");
            // add answer
            registers.change(zero, zero, zero, zero, one, registers.read_bytes, registers.current_value, registers.quotient_value, registers.exponent_value,
                             registers.odd_exponent, registers.r, registers.q_r, registers.decoded_bytes, registers.is_error, registers.gap, registers.final_value,
                             registers.read_address, registers.hash_lim, registers.hash_value, F::from_canonical_u64(bid_amount), F::from_canonical_u64(nonce),
                             F::from_canonical_u64(winner_amount), F::from_canonical_u64(change_winner), registers.winner_address);
        }
        values.push(registers.clone());


    }

    let height = values.len().next_power_of_two();
    registers.is_dummy = one;
    while values.len() < height {
        values.push(registers.clone());
    }

    let mut trace = RowMajorMatrix::new(values.iter().flat_map(|r| r.to_vec()).collect(), NUM_BID_COLS);
    let rows: &[BidCols<F>] = &values;
    trace
}

pub fn new_bidder<F: Field> (registers: &mut BidCols<F>, exponent: F, address_bytes: &[u8]) {
    let one = F::one();
    let zero = F::zero();
    let address: [F; 20] = address_bytes.iter().map(|e| F::from_canonical_u8(*e)).collect::<Vec<F>>().try_into().expect("slice with incorrect length");
    let (new_hash_value, new_hash_lim) = hash_address(registers.hash_value, address_bytes, registers.hash_lim);
    registers.change(zero, one, zero, zero, zero, [zero; READ_BYTES], zero, zero, exponent, zero, one,
                     zero, [zero; DECODED_BYTES], zero, one, zero, address, registers.hash_lim, new_hash_value,
                     zero, zero, registers.winner_amount, zero, registers.winner_address);
}

pub fn hash_address<F: Field> (hash_value: F, address: &[u8], hash_lim: F) -> (F, F) {
    let base = F::from_canonical_u64(BASE as u64);
    let mut new_hash_value = hash_value;
    let mut new_hash_lim = hash_lim;
    // each 32-hash
    for i in 0..5 {
        let le_bytes: [u8; 4] = address[i*4..i*4+4].try_into().expect("slice with incorrect length");
        let num = F::from_canonical_u32(u32::from_le_bytes(le_bytes));
        new_hash_value = new_hash_value + num * new_hash_lim;
        new_hash_lim = new_hash_lim * base;
    }
    (new_hash_value, new_hash_lim)
}