pub fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>()
}
pub fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, String> {
    if hex.len() % 2 != 0 {
        return Err("Hex string must have an even number of characters.".to_string());
    }

    (0..hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex[i..i + 2], 16)
                .map_err(|_| format!("Invalid hex character at index {}", i))
        })
        .collect()
}

/// Calculates Modular Exponent for given `base`, `exponent` and `modulus`.
#[must_use]
pub fn mod_pow(base: u64, exponent: u64, modulus: u64) -> u64 {

    let mut result = 1 as u128;
    let mut base_ = (base % modulus) as u128;
    let mut exp = exponent.clone() as u128;

    while !(exp == 0) {
        let s_bit = exp % 2;
        result = ((&result * &base_) % modulus as u128) * s_bit + &result * (1 - s_bit);

        exp >>= 1u8;
        base_ = (base_.pow(2)) % modulus as u128;
    }
    result = result % modulus as u128;
    result as u64
}