use heapless::String;
use num_enum::IntoPrimitive;

#[derive(Debug, PartialEq, Copy, Clone)]
#[derive(IntoPrimitive)]
#[repr(u8)]
pub enum ImprovError {
    None = 0x00,
    InvalidRpc = 0x01,
    UnknownRpc = 0x02,
    UnableToConnect = 0x03,
    NotAuthorized = 0x04,
    Unknown = 0xff
}


#[derive(IntoPrimitive, Copy, Clone)]
#[repr(u8)]
pub enum ImprovState {
    Stopped = 0x00,
    AwaitingAuthorization = 0x01,
    Authorized = 0x02,
    Provisioning = 0x03,
    Provisioned = 0x04
}

#[derive(Debug, PartialEq)]
#[repr(u8)]
enum ImprovCommandIdentifier {
    WifiSettings = 0x01,
    GetCurrentState = 0x02,
    GetDeviceInfo = 0x03,
    GetWifiNetworks = 0x04
}

impl TryFrom<u8> for ImprovCommandIdentifier {
    type Error = ImprovError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(ImprovCommandIdentifier::WifiSettings),
            0x02 => Ok(ImprovCommandIdentifier::GetCurrentState),
            0x03 => Ok(ImprovCommandIdentifier::GetDeviceInfo),
            0x04 => Ok(ImprovCommandIdentifier::GetWifiNetworks),
            _ => Err(ImprovError::UnknownRpc)
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ImprovCommand {
    WifiSettings { ssid: String<32>, password: String<64> },
    GetCurrentState,
    GetDeviceInfo,
    GetWifiNetworks
}

impl ImprovCommand {


    pub fn from_bytes(data: &[u8]) -> Result<ImprovCommand, ImprovError> {
        let cmd = ImprovCommandIdentifier::try_from(data[0])?;

        match cmd {
            ImprovCommandIdentifier::WifiSettings => {
                let ssid_length = data[2] as usize;
                let ssid_start = 3;
                let ssid_end= ssid_start + ssid_length;

                let pass_length = data[ssid_end] as usize;
                let pass_start = ssid_end + 1;
                let pass_end = pass_start + pass_length;

                let ssid = core::str::from_utf8(&data[ssid_start..ssid_end]).map(|x| String::from(x)).map_err(|_| ImprovError::InvalidRpc)?;
                let password = core::str::from_utf8(&data[pass_start..pass_end]).map(|x| String::from(x)).map_err(|_| ImprovError::InvalidRpc)?;

                Ok(ImprovCommand::WifiSettings { ssid, password })
            },
            ImprovCommandIdentifier::GetDeviceInfo => Ok(ImprovCommand::GetDeviceInfo),
            ImprovCommandIdentifier::GetCurrentState => Ok(ImprovCommand::GetCurrentState),
            ImprovCommandIdentifier::GetWifiNetworks => Ok(ImprovCommand::GetWifiNetworks),
        }
    }
}

pub const IMPROV_SERVICE_UUID: &str = "00467768-6228-2272-4663-277478268000";
pub const IMPROV_STATUS_UUID: &str = "00467768-6228-2272-4663-277478268001";
pub const IMPROV_ERROR_UUID: &str = "00467768-6228-2272-4663-277478268002";
pub const IMPROV_RPC_COMMAND_UUID: &str = "00467768-6228-2272-4663-277478268003";
pub const IMPROV_RPC_RESULT_UUID: &str = "00467768-6228-2272-4663-277478268004";
pub const IMPROV_CAPABILITIES_UUID: &str = "00467768-6228-2272-4663-277478268005";


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {


        let bytes: [u8; 32] = [0x01, 0x1e, 0x0c, 0x4d,0x79,0x57,0x69,0x72,0x65,0x6c,0x65,0x73,0x73,0x41,0x50, 0x10, 0x6d,0x79,0x73,0x65,0x63,0x75,0x72,0x65,0x70,0x61,0x73,0x73,0x77,0x6f,0x72,0x64];

        assert_eq!(ImprovCommand::from_bytes(&bytes), Ok(ImprovCommand::WifiSettings { ssid: String::from("MyWirelessAP"), password: String::from("mysecurepassword")}))
    }
}
