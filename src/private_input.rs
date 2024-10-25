use p3_field::Field;

#[derive(Clone)]
pub struct PrivateInput<F> {
    pub private_modulus: F,
    pub private_exponent: F,
}

impl <F> PrivateInput<F> {
    pub fn new(private_modulus: F, private_exponent: F) -> Self {
        Self {
            private_modulus,
            private_exponent
        }
    }
}