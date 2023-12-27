pub struct Config {
    pub stack_size: usize,
}

impl Default for Config {
    Config {
        stack_size: 1 << 14
    }
}
