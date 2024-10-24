pub const READ_BYTES: usize = 4;
pub const DECODED_BYTES: usize = 4;
pub const NUM_PROVER_COLS: usize = 19;
#[derive(Default, Clone, Debug)]
pub struct BidCols<T> {
    /// 8 bytes read
    pub is_reading: T,
    pub is_exponent: T,
    pub read_bytes: [T; READ_BYTES],
    // pub read_value: T,
    pub current_value: T,
    pub quotient_value: T,
    pub exponent_value: T,
    pub odd_exponent: T,
    pub r: T,
    pub q_r: T,
    pub decoded_bytes: [T; DECODED_BYTES],
    pub is_error: T,
    pub lim: T,
    pub final_value: T,
}

impl<T> BidCols<T> {
    pub fn to_vec(&self) -> Vec<T>
    where
        T: Clone,
    {
        let mut res: Vec<T> = vec![];
        let d = self.clone();
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
        res.push(d.lim);
        res.push(d.final_value);
        res
    }

    pub fn change(
        &mut self,
        is_reading: T,
        is_exponent: T,
        read_bytes: [T;READ_BYTES],
        // read_value: T,
        current_value: T,
        quotient_value: T,
        exponent_value: T,
        odd_exponent: T,
        r: T,
        q_r: T,
        decoded_bytes: [T;DECODED_BYTES],
        is_error: T,
        lim: T,
        final_value: T
    ) {
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
        self.lim = lim;
        self.final_value = final_value;
    }
}

