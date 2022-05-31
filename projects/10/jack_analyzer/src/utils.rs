pub struct Padding(usize);

impl Padding {
    pub fn new() -> Self {
        Padding(0)
    }

    pub fn to_spaces(&self) -> String {
        vec![" "; self.0].concat()
    }

    pub fn increment(&mut self) -> &Self {
        self.0 += 2;
        self
    }

    pub fn decrement(&mut self) -> &Self {
        self.0 -= 2;
        self
    }
}