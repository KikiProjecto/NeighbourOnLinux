pub struct Items {
    pub key_taken: bool,
    pub has_key: bool,
    pub carrying: bool,
}

impl Items {
    pub fn new() -> Self {
        Self {
            key_taken: false,
            has_key: false,
            carrying: false,
        }
    }
}
