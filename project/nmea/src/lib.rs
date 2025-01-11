pub struct Nmea {
    talker_id: TalkerId,
    sentence: Sentence,
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
    PxxxProprietary,
    PqQuectelQuirk,
    QzQzssRegionalGpsASJapan,
    SdDepthSounder,
    StSkytraq,
    TiTurnIndicator,
    YxTransducer,
    WiWeatherInstrument,
    NotRecognized,
}

struct Sentence {
    sentence_type: String,
}

impl Nmea {
    pub fn new() -> Self {
        Nmea {
            talker_id: TalkerId::NotRecognized,
            sentence: Sentence {
                sentence_type: String::new(),
            },
        }
    }

    /*
        TODO : parse talkerid
        TODO : parse sentences : [ HDT , VDM, VDO, GGA, GLL, RMC, DPT, VHW, TTM, TLL, ZDA]
        TODO : parse all other sentences or most of them anyway
    */

    pub fn parse(&self, _sentence: String) -> Result<(), i32> {
        Ok(())
    }
}
