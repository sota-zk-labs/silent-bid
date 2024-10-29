use core::borrow::{BorrowMut, Borrow};

pub const READ_BYTES: usize = 4;
pub const DECODED_BYTES: usize = 4;
pub const ADDRESS_BYTES: usize = 20;
pub const NUM_BID_COLS: usize = 68;
pub const AMOUNT_BITS: usize = 64;
pub const BASE: usize = 311;
#[derive(Clone, Debug)]
#[repr(C)]
pub struct BidCols<T> {
    pub is_dummy: T,
    pub new_bidder: T,
    pub is_reading: T,
    pub is_exponent: T,
    pub computing_winner: T,
    pub read_bytes: [T; READ_BYTES],
    pub current_value: T,
    pub quotient_value: T,
    pub exponent_value: T,
    pub odd_exponent: T,
    pub r: T,
    pub q_r: T,
    pub decoded_bytes: [T; DECODED_BYTES],
    pub is_error: T,
    pub gap: T,
    pub final_value: T,
    pub read_address: [T; ADDRESS_BYTES],
    pub hash_lim: T,
    pub hash_value: T,
    // logic
    pub bid_amount: T,
    pub nonce: T,
    // pub bid_amount_bits: [T; AMOUNT_BITS],
    pub winner_amount: T,
    // pub winner_amount_bits: [T; AMOUNT_BITS],
    // pub pos: T,
    pub change_winner: T,
    pub winner_address: [T; ADDRESS_BYTES],

}


impl<T: Default + std::marker::Copy> Default for BidCols<T> {
    fn default() -> Self {
        Self {
            is_dummy: T::default(),
            new_bidder: T::default(),
            is_reading: T::default(),
            is_exponent: T::default(),
            computing_winner: T::default(),
            read_bytes: [T::default(); READ_BYTES],
            current_value: T::default(),
            quotient_value: T::default(),
            exponent_value: T::default(),
            odd_exponent: T::default(),
            r: T::default(),
            q_r: T::default(),
            decoded_bytes: [T::default(); DECODED_BYTES],
            is_error: T::default(),
            gap: T::default(),
            final_value: T::default(),
            read_address: [T::default(); ADDRESS_BYTES],
            hash_lim: T::default(),
            hash_value: T::default(),
            bid_amount: T::default(),
            nonce: T::default(),
            // bid_amount_bits: [T::default(); AMOUNT_BITS],
            winner_amount: T::default(),
            // winner_amount_bits: [T::default(); AMOUNT_BITS],
            // pos: T::default(),
            change_winner: T::default(),
            winner_address: [T::default(); ADDRESS_BYTES],
        }
    }
}

impl<T> BidCols<T> {

    pub fn to_vec(&self) -> Vec<T>
    where
        T: Clone,
    {
        let mut res: Vec<T> = vec![];
        let d = self.clone();
        res.push(d.is_dummy);
        res.push(d.new_bidder);
        res.push(d.is_reading);
        res.push(d.is_exponent);
        res.push(d.computing_winner);
        res.extend(d.read_bytes.to_vec());
        // res.push(d.read_value);
        res.push(d.current_value);
        res.push(d.quotient_value);
        res.push(d.exponent_value);
        res.push(d.odd_exponent);
        res.push(d.r);
        res.push(d.q_r);
        res.extend(d.decoded_bytes.to_vec());
        res.push(d.is_error);
        res.push(d.gap);
        res.push(d.final_value);
        res.extend(d.read_address.to_vec());
        res.push(d.hash_lim);
        res.push(d.hash_value);
        res.push(d.bid_amount);
        res.push(d.nonce);
        // res.extend(d.bid_amount_bits.to_vec());
        res.push(d.winner_amount);
        // res.extend(d.winner_amount_bits);
        // res.push(d.pos);
        res.push(d.change_winner);
        res.extend(d.winner_address.to_vec());
        res
    }

    pub fn change(
        &mut self,
        is_dummy: T,
        new_bidder: T,
        is_reading: T,
        is_exponent: T,
        computing_winner: T,
        read_bytes: [T;READ_BYTES],
        current_value: T,
        quotient_value: T,
        exponent_value: T,
        odd_exponent: T,
        r: T,
        q_r: T,
        decoded_bytes: [T;DECODED_BYTES],
        is_error: T,
        gap: T,
        final_value: T,
        read_address: [T; ADDRESS_BYTES],
        hash_lim: T,
        hash_value: T,
        bid_amount: T,
        nonce: T,
        // bid_amount_bits: [T; AMOUNT_BITS],
        winner_amount: T,
        // winner_amount_bits: [T; AMOUNT_BITS],
        // pos: T,
        change_winner: T,
        winner_address: [T; ADDRESS_BYTES],
    ) {
        self.is_dummy = is_dummy;
        self.new_bidder = new_bidder;
        self.is_reading = is_reading;
        self.is_exponent = is_exponent;
        self.computing_winner = computing_winner;
        self.read_bytes = read_bytes;
        // self.read_value = read_value;
        self.current_value = current_value;
        self.quotient_value = quotient_value;
        self.exponent_value = exponent_value;
        self.odd_exponent = odd_exponent;
        self.r = r;
        self.q_r = q_r;
        self.decoded_bytes = decoded_bytes;
        self.is_error = is_error;
        self.gap = gap;
        self.final_value = final_value;
        self.read_address = read_address;
        self.hash_lim = hash_lim;
        self.hash_value = hash_value;
        self.bid_amount = bid_amount;
        self.nonce = nonce;
        // self.bid_amount_bits = bid_amount_bits;
        self.winner_amount = winner_amount;
        // self.winner_amount_bits = winner_amount_bits;
        // self.pos = pos;
        self.change_winner = change_winner;
        self.winner_address = winner_address;
    }
}

impl<T> Borrow<BidCols<T>> for [T] {
    fn borrow(&self) -> &BidCols<T> {
        debug_assert_eq!(self.len(), NUM_BID_COLS);
        let (prefix, shorts, suffix) = unsafe { self.align_to::<BidCols<T>>() };
        debug_assert!(prefix.is_empty(), "Alignment should match");
        debug_assert!(suffix.is_empty(), "Alignment should match");
        debug_assert_eq!(shorts.len(), 1);
        &shorts[0]
    }
}

impl<T> BorrowMut<BidCols<T>> for [T] {
    fn borrow_mut(&mut self) -> &mut BidCols<T> {
        debug_assert_eq!(self.len(), NUM_BID_COLS);
        let (prefix, shorts, suffix) = unsafe { self.align_to_mut::<BidCols<T>>() };
        debug_assert!(prefix.is_empty(), "Alignment should match");
        debug_assert!(suffix.is_empty(), "Alignment should match");
        debug_assert_eq!(shorts.len(), 1);
        &mut shorts[0]
    }
}