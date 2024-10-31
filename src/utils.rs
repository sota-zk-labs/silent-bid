
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

pub fn address_to_bytes(address: &str) -> Vec<u8> {
    let address = address.trim_start_matches("0x");

    // Decode hex string to a vector of bytes
    
    hex::decode(address).unwrap()
}

pub fn bytes_to_address(address_bytes: &[u8]) -> String {
    // Convert bytes to a hex string
    let address_hex = hex::encode(address_bytes);

    // Add "0x" prefix to make it look like a typical Ethereum address
    let ethereum_address = format!("0x{}", address_hex);
    ethereum_address
}
