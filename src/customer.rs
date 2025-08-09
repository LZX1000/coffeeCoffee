use crate::Config; // Used for rng chances

#[derive(Debug, Clone)]
pub struct Customer(u8);

impl Customer {
    //Bit masks                      (0b0000_0000)
    const DESIRED_DRINK_ID_MASK: u8 = 0b0000_1111;
    const RICH_FLAG            : u8 = 0b0001_0000;

    pub fn new() -> Self {
        Customer(0)
    }

    // Getters
    pub fn desired_drink(&self) -> u8 {
        self.0 & Self::DESIRED_DRINK_ID_MASK
    }

    // Setters
    pub fn set_desired_drink(&mut self, id: u8) {
        self.0 = (self.0 & !Self::DESIRED_DRINK_ID_MASK) | (id & 0b1111)
    }
}