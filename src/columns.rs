use core::borrow::{BorrowMut, Borrow};

pub const READ_BYTES: usize = 4;
pub const DECODED_BYTES: usize = 4;
pub const ADDRESS_BYTES: usize = 20;
pub const NUM_BID_COLS: usize = 43;

pub const BASE: usize = 311;
#[derive(Default, Clone, Debug)]
#[repr(C)]
pub struct BidCols<T> {
    pub is_dummy: T,
    pub new_bidder: T,
    pub is_reading: T,
    pub is_exponent: T,
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
        res
    }

    pub fn change(
        &mut self,
        is_dummy: T,
        new_bidder: T,
        is_reading: T,
        is_exponent: T,
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
        hash_value: T
    ) {
        self.is_dummy = is_dummy;
        self.new_bidder = new_bidder;
        self.is_reading = is_reading;
        self.is_exponent = is_exponent;
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