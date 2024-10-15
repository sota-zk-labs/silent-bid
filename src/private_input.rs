#[derive(Clone)]
pub struct PrivateInput {
    private_key: String,
}

impl PrivateInput {
    pub fn new(private_key: String) -> Self {
        Self {
            private_key
        }
    }
}