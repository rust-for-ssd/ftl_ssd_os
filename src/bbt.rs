pub struct BadBlockTable {
    // Channels: &'static[u8]
    Channels: [u8;2]
}


impl BadBlockTable {
    pub fn new() -> Self {
        BadBlockTable {
            Channels: [42, 42] 
        }
    }
    pub fn read(&self) -> u8 {
        self.Channels[0]
    }
}
