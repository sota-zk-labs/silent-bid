# Plonky3 Proving Service Implementations

## Overview

Plonky3 settings:

- **Field**: Goldilocks
- **Hash**: Keccak256Hash
- **PCS**: TwoAdicFriPcs
- **FRI config**: 
	- log_blowup: 3
	- num_queries: 80,  
	- proof_of_work_bits: 16,  
Public input:
- Modulo for RSA
- Hash value
- Base value for Rolling Hash

## Decryption

### Algorithm

We use a 32-bit RSA scheme for encryption and decryption. The owner's public key consists of an exponent $e$ and a modulus $n$, while the private key is the exponent $d$.

To encrypt a message $m$, the bidder first parses it into a vector of bytes, then groups consecutive pair of bytes and transforms them into a 16-bit number $y$. The encryption is computed as $Enc(y) = y^e \mod n$. 

To decrypt the bid amount, the proving service reads every 4 consecutive bytes, transforms them into a 32-bit number $x$, and computes the decryption as $Dec(x) = x^d \mod n$.

### Execution Trace

For the decryption process, we uses the following flags:
- `new_bidder`: Activated when starting to decrypt a new bidder’s amount.
- `is_reading`: Activated when reading a new set of 4 encrypted bytes.
- `is_exponent`: Activated when beginning the exponentiation computation.
- `is_error`: Activated when decryption fails.

We store the 4 encrypted bytes in `read_bytes` and theirs integer value in `current_value`. The value $d$ is stored in `exponent_value`. 

During decryption, we compute $x^d$  using the method:
$$x^d = (x^2)^{d/2} * x^{d \mod 2}$$

For example, to compute $2^7 \mod 11$, the execution trace will be:

| current value | quotient value | exponent value | odd exponent | r   | q_r |
| ------------- | -------------- | -------------- | ------------ | --- | --- |
| 2             | 0              | 7              | 0            | 1   | 0   |
| 4             | 0              | 3              | 1            | 2   | 0   |
| 5             | 1              | 1              | 1            | 8   | 0   |
| 7             | 3              |                |              |     |     |

The final value of `current value` is $7$, which results from $2^7 = 128 \mod 11$. We will outline how the exponentiation process works.

First, the exponent $d$ is divided by $2$. If $d$ is odd, then the `odd exponent` is set to $1$ and 
$r = r * \text{pre current value}$. Otherwise, `odd exponent` is set to 0 and `r` remains unchanged. In each iteration, `current value` is squared ($\text{current value} = \text{pre current value}^2$), while $\text{quotient value} = \text{pre current value}^2 \div n$.  

The purpose of the `quotient value` column is tied to Plonky3’s constraints, which only support addition, subtraction, and multiplication. To define the constraints of the modulo operation, we use `quotient_value` to check: $\text{pre current value}^2 = \text{quotient value} * n + \text{current value}$. 
The reasoning applies similarly to `q_r`. When `exponent_value` finally reaches 1, the answer is given by $\text{current value} = \text{pre current value} * r$.

Since the plain bid amount is a 64-bit number (with the last 3 digits as the nonce), we can reconstruct it from the decrypted value of each 4-byte segment:

`final_value += current_value * gap;`

where `gap` is $2^{16}$. This factor accounts for encryption of each pair of bytes, so reconstructing the original amount requires multiplying by $2^{8*2} = 2^{16}$.
### Constraints

There are several conditions to satisfy:
- **Reading Phase**: During the reading phase, `current_value` should be constructed from `read_bytes` as:
	`builder.when(is_reading).assert_eq(local.current_value,  local.read_bytes[0] + local.read_bytes[1] * lim1 + local.read_bytes[2] * lim2 + local.read_bytes[3] * lim3);` where `lim` is power of $2^{16}$.
- **Exponent Halving**: The `exponent_value` is halved for each row, given by: `builder.when(next_exponent).assert_eq(local.exponent_value, next.exponent_value * two + next.odd_exponent);`
- **Doubling of `current_value`**: For each row, `current_value` doubles, verified by: `builder.when(next_exponent).assert_eq(local.current_value * local.current_value, next.quotient_value * modules.clone() + next.current_value);`
- **Modulo Operation**: The correctness of the modulo operation is confirmed by: `builder.when(next_odd_exponent).assert_eq(local.r * local.current_value, next.q_r * modules.clone() + next.r);`
- **Final Value**: The final accumulated value must satisfy: `assert_eq(next.final_value, local.final_value + next.current_value * local.gap);`

## Hashing

### Algorithm
We use a Rolling Hash for the hash function. The algorithm operates simply: the proving service reads each 4-byte segment from the encrypted amount, transforms it into an integer $x$, and adds $x * base^k$ to the hash result. The `base` value is defined by the owner and specified in the smart contract during the initial setup.

### Execution Trace

Beside the flags, this process includes some columns:
- `hash_lim`: the value of $base^k$
- `hash_value`: the value of the final hash

### Constraints

During the reading phase, the hash parameters are updated as follows:
- Hash lim: The hash lim is scaled by the base with each read: $\text{new hash lim} = \text{hash lim} * base$
- Hash value: The hash value is also updated based on the current value: $\text{new hash value} = \text{hash value} + \text{current value} * \text{hash lim}$. 
	
	
Additionally, to ensure the integrity of the bid amount with the bidder’s address, the proving service hashes the bid address. Given that an Ethereum address is 20 bytes, it is divided into 5 parts, and each part is hashed.

Thus, we have four main constraints:

- The hash value must be updated when reading bytes or when starting to decrypt a new bidder:
```
for i in 0..5 {  
    let hash_num = next.read_address[i*4] +  next.read_address[i*4 + 1] * AB::Expr::from_canonical_u64(256)  + local.read_address[i*4 + 2] * AB::Expr::from_canonical_u64(65536) + next.read_address[i*4 + 3] * AB::Expr::from_canonical_u64(16777216);  
    new_hash += hash_num * start_lim.clone();  
    start_lim = base.clone() * start_lim;  
}
```
- **Hash Lim Update**: The hash limit (`hash_lim`) must be updated after reading a new set of bytes or after reading a new bidder address: `assert_eq(local.hash_lim * base.clone(), next.hash_lim);`
- **Hash Value Calculation**: The hash must be computed correctly based on the updated `hash_lim` and `current_value`:  `assert_eq(local.hash_value + next.hash_lim * next.current_value, next.hash_value);`
- **Output Hash Verification**: The computed hash value should match the public input hash to ensure integrity:
```
let final_hash = builder.public_values()[1];  
builder.when_last_row().assert_eq(local.hash_value, final_hash);
```








