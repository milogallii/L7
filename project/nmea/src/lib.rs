pub struct Nmea {
    talker_id: String,
}

impl Nmea {
    pub fn new() -> Self {
        Nmea {
            talker_id: String::new(),
        }
    }

    pub fn parse(&self, sentence: String) -> Result<(), i32> {}
}
