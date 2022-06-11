extern crate hidapi;

const VENDOR_ID: u16 = 0x4FE;

#[repr(u8)]
#[derive(Debug)]
enum KeyboardMode {
    Hhk = 0,
    Mac = 1,
    Light = 2,
}

#[derive(Debug)]
#[allow(dead_code)]
struct KeyboardInfo {
    type_number: String,
    revision: String,
    serial: String,
    app_firm_version: String,
    boot_firm_version: String,
    running_firmware: RunningFirmware,
}

impl std::convert::TryFrom<u8> for KeyboardMode {
    type Error = ();

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            x if x == KeyboardMode::Hhk as u8 => Ok(KeyboardMode::Hhk),
            x if x == KeyboardMode::Mac as u8 => Ok(KeyboardMode::Mac),
            x if x == KeyboardMode::Light as u8 => Ok(KeyboardMode::Light),
            _ => Err(()),
        }
    }
}

#[repr(u8)]
#[derive(Debug)]
enum RunningFirmware {
    AppFirmware = 0,
    BootFirmware = 1,
}

impl std::convert::TryFrom<u8> for RunningFirmware {
    type Error = ();

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            x if x == RunningFirmware::AppFirmware as u8 => Ok(RunningFirmware::AppFirmware),
            x if x == RunningFirmware::BootFirmware as u8 => Ok(RunningFirmware::BootFirmware),
            _ => Err(()),
        }
    }
}


struct HhkB {
    dev: hidapi::HidDevice,
}

fn main() {
    let api = hidapi::HidApi::new().unwrap();

    use std::collections::HashSet;

    let products = api
        .device_list()
        .filter(|d| d.vendor_id() == VENDOR_ID)
        .map(|d| d.product_id())
        .collect::<HashSet<_>>();

    // todo check product_id
    let product_id = products.into_iter().next().unwrap();

    let dev = api.open(VENDOR_ID, product_id).unwrap();

    let kb = HhkB { dev };

    println!("dips: {:?}", kb.dips());

    println!("mode: {:?}", kb.mode());

    println!("info: {:#?}", kb.info());
}

impl HhkB {
    /// Send `msg` and return the response if `prefix` is a prefix of the response
    fn send(&self, msg: &[u8], prefix: &[u8]) -> Option<[u8; 64]> {
        let mut output = [0u8; 64];
        output[..msg.len()].copy_from_slice(msg);

        let res = self.dev.write(&output).unwrap();
        assert_eq!(res, 65);

        let mut buf = [0u8; 64];
        let res = self.dev.read(&mut buf[..]).unwrap();
        assert_eq!(res, 64);

        let veri = buf
            .iter()
            .zip(prefix.iter())
            .map(|(a, b)| a == b)
            .reduce(|last, new| if !last { false } else { new })
            .unwrap();

        if veri {
            Some(buf)
        } else {
            None
        }
    }

    pub fn dips(&self) -> [bool; 6] {
        let buf = self
            .send(&[0x00, 0xAA, 0xAA, 0x05], &[85, 85, 5, 0, 0, 12])
            .unwrap();

        let mut out = [false; 6];
        out[..].copy_from_slice(&buf[6..12].iter().map(|v| v == &1).collect::<Vec<_>>());

        out
    }

    pub fn mode(&self) -> KeyboardMode {
        let buf = self
            .send(&[0x00, 0xAA, 0xAA, 0x06], &[85, 85, 6, 0, 0, 1])
            .unwrap();

        buf[6].try_into().unwrap()
    }

    fn info(&self) -> KeyboardInfo {
        let buf = self
            .send(&[0x00, 0xAA, 0xAA, 0x02], &[85, 85, 2, 0, 0 /*, 48*/])
            .unwrap();

        fn s_enc(vec: Vec<u8>) -> String {
            String::from_utf8(vec).unwrap().replace(char::from(0), "")
        }

        fn ver_enc(vec: &[u8]) -> String {
            format!("{:X}{}.{}{}", vec[0], vec[1], vec[2], vec[3])
        }

        let type_number = s_enc(buf[6..26].to_vec());
        let revision = s_enc(buf[26..30].to_vec());
        let serial = s_enc(buf[30..46].to_vec());

        // ignore last 4 bytes
        let app_firm_version = ver_enc(&buf[46..50]); // 46..54
        let boot_firm_version = ver_enc(&buf[54..58]); // 54..62

        let running_firmware = buf[62].try_into().unwrap();

        KeyboardInfo {
            type_number,
            revision,
            serial,
            app_firm_version,
            boot_firm_version,
            running_firmware,
        }
    }
}
