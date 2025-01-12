use hashbrown::HashMap;

pub struct Nmea {
    talker_id: TalkerId,
    sentence_type: SentenceType,
    sentence_fields: hashbrown::HashMap<String, String>,
}

enum TalkerId {
    AiAlarmIndicator,
    ApAutoPilot,
    BdBeidouChina,
    CdDsc,
    EcEcdis,
    GaGalileoPs,
    GbBeidouChina,
    GiNavicIrnssIndia,
    GlGlonassIEIC611621,
    GnMultipleSatelliteSystem,
    GpGlobalPositioningSystemReceiver,
    GqQZSSRegionalGpsASJapan,
    HcHeadingCompass,
    HeGyroNorthSeeking,
    IiIntegratedInstrumentation,
    InIntegratedNavigation,
    LcLorancReceiver,
    PqQuectelQuirk,
    QzQzssRegionalGpsASJapan,
    SdDepthSounder,
    StSkytraq,
    TiTurnIndicator,
    YxTransducer,
    WiWeatherInstrument,
    NotRecognized,
}

enum SentenceType {
    // Actual vessel heading in degrees true produced by any device or system producing true heading.
    Hdt,
    // TODO Get informations about these two
    Vdm,
    Vdo,
    // This ones are sentences commonly emitted by GPS units.
    // Time, Position and fix related data for a GPS receiver.
    Gga,
    Gll,
    Rmc,
    Zda,
    // Depth of water
    Dpt,
    // Water speed and heading
    Vhw,
    // Tracked target message
    Ttm,
    // Target latitude and longitude
    Tll,
    NotRecognized,
}

impl Nmea {
    fn default() -> Self {
        Nmea {
            talker_id: TalkerId::NotRecognized,
            sentence_type: SentenceType::NotRecognized,
            sentence_fields: hashbrown::HashMap::new(),
        }
    }

    pub fn parse(&mut self, sentence: String) -> Result<(), i32> {
        self.talker_id = self.parse_talker_id(&sentence);
        let (sentence_type, sentence_fields) = self.parse_sentence(&sentence);
        self.sentence_type = sentence_type;
        self.sentence_fields = sentence_fields;
        Ok(())
    }

    fn parse_talker_id(&mut self, sentence: &str) -> TalkerId {
        let talker_id_0 = sentence.chars().nth(1);
        let talker_id_1 = sentence.chars().nth(2);

        match (talker_id_0, talker_id_1) {
            (Some('A'), Some('I')) => TalkerId::AiAlarmIndicator,
            (Some('A'), Some('P')) => TalkerId::ApAutoPilot,
            (Some('B'), Some('D')) => TalkerId::BdBeidouChina,
            (Some('C'), Some('D')) => TalkerId::CdDsc,
            (Some('E'), Some('C')) => TalkerId::EcEcdis,
            (Some('G'), Some('A')) => TalkerId::GaGalileoPs,
            (Some('G'), Some('B')) => TalkerId::GbBeidouChina,
            (Some('G'), Some('I')) => TalkerId::GiNavicIrnssIndia,
            (Some('G'), Some('L')) => TalkerId::GlGlonassIEIC611621,
            (Some('G'), Some('N')) => TalkerId::GnMultipleSatelliteSystem,
            (Some('G'), Some('P')) => TalkerId::GpGlobalPositioningSystemReceiver,
            (Some('G'), Some('Q')) => TalkerId::GqQZSSRegionalGpsASJapan,
            (Some('H'), Some('C')) => TalkerId::HcHeadingCompass,
            (Some('H'), Some('E')) => TalkerId::HeGyroNorthSeeking,
            (Some('I'), Some('I')) => TalkerId::IiIntegratedInstrumentation,
            (Some('I'), Some('N')) => TalkerId::InIntegratedNavigation,
            (Some('L'), Some('C')) => TalkerId::LcLorancReceiver,
            (Some('P'), Some('Q')) => TalkerId::PqQuectelQuirk,
            (Some('Q'), Some('Z')) => TalkerId::QzQzssRegionalGpsASJapan,
            (Some('S'), Some('D')) => TalkerId::SdDepthSounder,
            (Some('S'), Some('T')) => TalkerId::StSkytraq,
            (Some('T'), Some('I')) => TalkerId::TiTurnIndicator,
            (Some('Y'), Some('X')) => TalkerId::YxTransducer,
            (Some('W'), Some('I')) => TalkerId::WiWeatherInstrument,
            _ => TalkerId::NotRecognized,
        }
    }

    fn parse_sentence(&self, sentence: &str) -> (SentenceType, hashbrown::HashMap<String, String>) {
        let sentence_type_0 = sentence.chars().nth(3);
        let sentence_type_1 = sentence.chars().nth(4);
        let sentence_type_2 = sentence.chars().nth(5);

        match (sentence_type_0, sentence_type_1, sentence_type_2) {
            (Some('H'), Some('D'), Some('T')) => (SentenceType::Hdt, self.parse_hdt(sentence)),
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
