use hmcdk::prelude::*;

pub type Arg = Vec<u8>;
pub type Args = Vec<Arg>;

#[derive(Default)]
pub struct ArgsBuilder {
    values: Args,
}

impl ArgsBuilder {
    pub fn new() -> Self {
        ArgsBuilder{values: Args::default()}
    }

    pub fn push<T: ToBytes>(&mut self, arg: T) {
        self.values.push(arg.to_bytes());
    }

    pub fn push_bytes(&mut self, arg: &[u8]) {
        self.values.push(arg.to_vec());
    }

    pub fn convert_to_vec(self) -> Vec<Vec<u8>> {
        self.values
    }
}
