use hashbrown::HashMap;

pub struct Nmea {
    pub talker_id: TalkerId,
    pub sentence_type: SentenceType,
    pub sentence_fields: hashbrown::HashMap<String, String>,
}

enum TalkerId {
    AiAlarmIndicator(String),
    ApAutoPilot(String),
    BdBeidouChina(String),
    CdDsc(String),
    EcEcdis(String),
    GaGalileoPs(String),
    GbBeidouChina(String),
    GiNavicIrnssIndia(String),
    GlGlonassIEIC611621(String),
    GnMultipleSatelliteSystem(String),
    GpGlobalPositioningSystemReceiver(String),
    GqQZSSRegionalGpsASJapan(String),
    HcHeadingCompass(String),
    HeGyroNorthSeeking(String),
    IiIntegratedInstrumentation(String),
    InIntegratedNavigation(String),
    LcLorancReceiver(String),
    PqQuectelQuirk(String),
    QzQzssRegionalGpsASJapan(String),
    SdDepthSounder(String),
    StSkytraq(String),
    TiTurnIndicator(String),
    YxTransducer(String),
    WiWeatherInstrument(String),
    NotRecognized,
}

enum SentenceType {
    // Actual vessel heading in degrees true produced by any device or system producing true heading.
    Hdt(String),
    // TODO Get informations about these two
    Vdm(String),
    Vdo(String),
    // This ones are sentences commonly emitted by GPS units.
    // Time, Position and fix related data for a GPS receiver.
    Gga(String),
    Gll(String),
    Rmc(String),
    Zda(String),
    // Depth of water
    Dpt(String),
    // Water speed and heading
    Vhw(String),
    // Tracked target message
    Ttm(String),
    // Target latitude and longitude
    Tll(String),
    NotRecognized,
}

impl Nmea {
    pub fn new() -> Self {
        Nmea {
            talker_id: TalkerId::NotRecognized,
            sentence_type: SentenceType::NotRecognized,
            sentence_fields: hashbrown::HashMap::new(),
        }
    }

    pub fn parse(&mut self, sentence: String) -> Result<(), i32> {
        let talker_id = self.parse_talker_id(&sentence);
        match talker_id {
            TalkerId::NotRecognized => Err(-1),
            _ => {
                self.talker_id = talker_id;
                let parse_sentence_result = self.parse_sentence(&sentence);
                match parse_sentence_result {
                    (SentenceType::NotRecognized, _) => Err(-1),
                    (sentence_type, sentence_fields) => {
                        self.sentence_type = sentence_type;
                        self.sentence_fields = sentence_fields;
                        Ok(())
                    }
                }
            }
        }
    }

    pub fn show(&self) {
        println!("[ TALKER ID : {}]", self.str_talker_id());
        println!("[ SENTENCE TYPE : {}]", self.str_sentence_type());
        self.sentence_fields
            .iter()
            .for_each(|(key, value)| println!("[ {} -- {} ]", key, value));
    }

    pub fn str_talker_id(&self) -> &str {
        match &self.talker_id {
            TalkerId::AiAlarmIndicator(value) => value,
            TalkerId::ApAutoPilot(value) => value,
            TalkerId::BdBeidouChina(value) => value,
            TalkerId::CdDsc(value) => value,
            TalkerId::EcEcdis(value) => value,
            TalkerId::GaGalileoPs(value) => value,
            TalkerId::GbBeidouChina(value) => value,
            TalkerId::GiNavicIrnssIndia(value) => value,
            TalkerId::GlGlonassIEIC611621(value) => value,
            TalkerId::GnMultipleSatelliteSystem(value) => value,
            TalkerId::GpGlobalPositioningSystemReceiver(value) => value,
            TalkerId::GqQZSSRegionalGpsASJapan(value) => value,
            TalkerId::HcHeadingCompass(value) => value,
            TalkerId::HeGyroNorthSeeking(value) => value,
            TalkerId::IiIntegratedInstrumentation(value) => value,
            TalkerId::InIntegratedNavigation(value) => value,
            TalkerId::LcLorancReceiver(value) => value,
            TalkerId::PqQuectelQuirk(value) => value,
            TalkerId::QzQzssRegionalGpsASJapan(value) => value,
            TalkerId::SdDepthSounder(value) => value,
            TalkerId::StSkytraq(value) => value,
            TalkerId::TiTurnIndicator(value) => value,
            TalkerId::YxTransducer(value) => value,
            TalkerId::WiWeatherInstrument(value) => value,
            _ => "",
        }
    }

    pub fn str_sentence_type(&self) -> &str {
        match &self.sentence_type {
            SentenceType::Hdt(value) => value,
            SentenceType::Vdm(value) => value,
            SentenceType::Vdo(value) => value,
            SentenceType::Gga(value) => value,
            SentenceType::Gll(value) => value,
            SentenceType::Rmc(value) => value,
            SentenceType::Zda(value) => value,
            SentenceType::Dpt(value) => value,
            SentenceType::Vhw(value) => value,
            SentenceType::Ttm(value) => value,
            SentenceType::Tll(value) => value,
            _ => "",
        }
    }

    fn parse_talker_id(&mut self, sentence: &str) -> TalkerId {
        let talker_id_0 = sentence.chars().nth(1);
        let talker_id_1 = sentence.chars().nth(2);

        match (talker_id_0, talker_id_1) {
            (Some('A'), Some('I')) => TalkerId::AiAlarmIndicator(String::from("AI")),
            (Some('A'), Some('P')) => TalkerId::ApAutoPilot(String::from("AP")),
            (Some('B'), Some('D')) => TalkerId::BdBeidouChina(String::from("BD")),
            (Some('C'), Some('D')) => TalkerId::CdDsc(String::from("CD")),
            (Some('E'), Some('C')) => TalkerId::EcEcdis(String::from("EC")),
            (Some('G'), Some('A')) => TalkerId::GaGalileoPs(String::from("GA")),
            (Some('G'), Some('B')) => TalkerId::GbBeidouChina(String::from("GB")),
            (Some('G'), Some('I')) => TalkerId::GiNavicIrnssIndia(String::from("GI")),
            (Some('G'), Some('L')) => TalkerId::GlGlonassIEIC611621(String::from("GL")),
            (Some('G'), Some('N')) => TalkerId::GnMultipleSatelliteSystem(String::from("GN")),
            (Some('G'), Some('P')) => {
                TalkerId::GpGlobalPositioningSystemReceiver(String::from("GP"))
            }
            (Some('G'), Some('Q')) => TalkerId::GqQZSSRegionalGpsASJapan(String::from("GQ")),
            (Some('H'), Some('C')) => TalkerId::HcHeadingCompass(String::from("HC")),
            (Some('H'), Some('E')) => TalkerId::HeGyroNorthSeeking(String::from("HE")),
            (Some('I'), Some('I')) => TalkerId::IiIntegratedInstrumentation(String::from("II")),
            (Some('I'), Some('N')) => TalkerId::InIntegratedNavigation(String::from("IN")),
            (Some('L'), Some('C')) => TalkerId::LcLorancReceiver(String::from("LC")),
            (Some('P'), Some('Q')) => TalkerId::PqQuectelQuirk(String::from("PQ")),
            (Some('Q'), Some('Z')) => TalkerId::QzQzssRegionalGpsASJapan(String::from("QZ")),
            (Some('S'), Some('D')) => TalkerId::SdDepthSounder(String::from("SD")),
            (Some('S'), Some('T')) => TalkerId::StSkytraq(String::from("ST")),
            (Some('T'), Some('I')) => TalkerId::TiTurnIndicator(String::from("TI")),
            (Some('Y'), Some('X')) => TalkerId::YxTransducer(String::from("YX")),
            (Some('W'), Some('I')) => TalkerId::WiWeatherInstrument(String::from("WI")),
            _ => TalkerId::NotRecognized,
        }
    }

    fn parse_sentence(&self, sentence: &str) -> (SentenceType, hashbrown::HashMap<String, String>) {
        let sentence_type_0 = sentence.chars().nth(3);
        let sentence_type_1 = sentence.chars().nth(4);
        let sentence_type_2 = sentence.chars().nth(5);

        match (sentence_type_0, sentence_type_1, sentence_type_2) {
            (Some('H'), Some('D'), Some('T')) => (
                SentenceType::Hdt(String::from("HDT")),
                self.parse_hdt(sentence),
            ),
            // (Some('V'), Some('D'), Some('M')) => (SentenceType::Vdm, self.parse_vdm(sentence)),
            // (Some('V'), Some('D'), Some('O')) => (SentenceType::Vdo, self.parse_vdo(sentence)),
            // (Some('G'), Some('G'), Some('A')) => (SentenceType::Gga, self.parse_gga(sentence)),
            // (Some('G'), Some('L'), Some('L')) => (SentenceType::Gll, self.parse_gll(sentence)),
            // (Some('R'), Some('M'), Some('C')) => (SentenceType::Rmc, self.parse_rmc(sentence)),
            // (Some('D'), Some('P'), Some('T')) => (SentenceType::Dpt, self.parse_dpt(sentence)),
            // (Some('V'), Some('H'), Some('W')) => (SentenceType::Vhw, self.parse_vhw(sentence)),
            // (Some('T'), Some('T'), Some('M')) => (SentenceType::Ttm, self.parse_ttm(sentence)),
            // (Some('T'), Some('L'), Some('L')) => (SentenceType::Tll, self.parse_tll(sentence)),
            // (Some('Z'), Some('D'), Some('A')) => (SentenceType::Zda, self.parse_zda(sentence)),
            _ => (SentenceType::NotRecognized, hashbrown::HashMap::new()),
        }
    }

    fn parse_hdt(&self, sentence: &str) -> hashbrown::HashMap<String, String> {
        let sentence_split: Vec<&str> = sentence.split(',').map(|s| s.trim()).collect();
        let heading_degrees = sentence_split.get(1);
        let true_checksum = sentence_split.get(2);

        match (heading_degrees, true_checksum) {
            (Some(v1), Some(v2)) => {
                let mut sentence_fields = hashbrown::HashMap::new();
                sentence_fields.insert(String::from("heading_degrees"), v1.to_string());
                sentence_fields.insert(String::from("true_checksum"), v2.to_string());
                sentence_fields
            }

            _ => HashMap::new(),
        }
    }

    // fn parse_vdm(&self, sentence: &str) -> hashbrown::HashMap<String, String> {}
    // fn parse_vdo(&self, sentence: &str) -> hashbrown::HashMap<String, String> {}
    // fn parse_gga(&self, sentence: &str) -> hashbrown::HashMap<String, String> {}
    // fn parse_gll(&self, sentence: &str) -> hashbrown::HashMap<String, String> {}
    // fn parse_rmc(&self, sentence: &str) -> hashbrown::HashMap<String, String> {}
    // fn parse_dpt(&self, sentence: &str) -> hashbrown::HashMap<String, String> {}
    // fn parse_vhw(&self, sentence: &str) -> hashbrown::HashMap<String, String> {}
    // fn parse_ttm(&self, sentence: &str) -> hashbrown::HashMap<String, String> {}
    // fn parse_tll(&self, sentence: &str) -> hashbrown::HashMap<String, String> {}
    // fn parse_zda(&self, sentence: &str) -> hashbrown::HashMap<String, String> {}
}
