#[derive(Debug, Clone)]
pub struct Customer(u8);

impl Customer {
    //Bit masks                         (0b0000_0000)
    const WAIT_TIME_TOLERANCE_MASK: u8 = 0b0000_1111;
    const DESIRED_DRINK_ID_MASK   : u8 = 0b1111_0000;

    pub fn new() -> Self {
        Customer(0)
    }

    // Getters
    pub fn wait_time(&self) -> u8 {
        self.0 & Self::WAIT_TIME_TOLERANCE_MASK
    }
    pub fn desired_drink(&self) -> u8 {
        (self.0 & Self::DESIRED_DRINK_ID_MASK) >> 4
    }

    // Setters
    pub fn set_wait_time(&mut self, time: u8) {
        self.0 = (self.0 & !Self::WAIT_TIME_TOLERANCE_MASK) | (time & 0b1111)
    }
    pub fn set_desired_drink(&mut self, id: u8) {
        self.0 = (self.0 & !Self::DESIRED_DRINK_ID_MASK) | ((id & 0b1111) << 4)
    }
}