use heapless::String;
use serde::Serialize;
use serde_json_core::ser::Error;

#[derive(Serialize)]
pub struct HaConfig {
    pub name: String<64>,
    pub stat_t: String<64>,
    pub unit_of_meas: String<3>,
    pub dev_class: Option<String<32>>,
    pub frc_upd: bool,
    pub val_tpl: String<64>
}

impl HaConfig {
    pub fn as_json(&self, buf: &mut [u8]) -> Result<usize, Error> {
        serde_json_core::to_slice(self, buf)
    }
}