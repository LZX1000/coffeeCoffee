#[derive(Debug, Clone)]
pub struct Player(u8);

impl Player {
    //Bit masks                      (0b0000_0000)
    const LEVEL_MASK           : u8 = 0b0000_0011;
    const CUSTOMERS_SERVED_MASK: u8 = 0b1111_1100; // Up to 63

    pub fn new() -> Self {
        Player(0)
    }

    // Getters
    pub fn level(&self) -> u8 {
        self.0 & Self::LEVEL_MASK
    }
    pub fn customers_served(&self) -> u8 {
        (self.0 & Self::CUSTOMERS_SERVED_MASK) >> 6
    }

    // // Setters
    // pub fn set_level(&mut self, time: u8) {
    //     self.0 = (self.0 & !Self::LEVEL_MASK) | (time & 0b1111)
    // }
    // pub fn set_customers_served(&mut self, count: u8) {
    //     self.0 = (self.0 & !Self::CUSTOMERS_SERVED_MASK) | ((count & 0b1111) << 6)
    // }
}