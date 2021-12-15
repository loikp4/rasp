use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize)]
pub struct usb_2jcie {
    tempureture: Option<f64>,
    relative_humidity: Option<f64>,
    ambient_light: Option<i16>,
    barometric_pressure: Option<f64>,
    sound_noise: Option<f64>,
    eTVOC: Option<u16>,
    eCO2: Option<u16>,
    disconfort_index: Option<f64>,
}

impl usb_2jcie {
    pub fn new() -> usb_2jcie {
        usb_2jcie {
            tempureture: None,
            relative_humidity: None,
            ambient_light: None,
            barometric_pressure: None,
            sound_noise: None,
            eTVOC: None,
            eCO2: None,
            disconfort_index: None,
        }
    }

    pub fn parse(&mut self, byte: &[u8]) {
        self.tempureture = Some(read_i16(byte[9], byte[8]) as f64 / 100.0);
        self.relative_humidity = Some(read_i16(byte[11], byte[10]) as f64 / 100.0);
        self.ambient_light = Some(read_i16(byte[13], byte[12]) as i16);
        self.barometric_pressure =
            Some(read_i32(byte[17], byte[16], byte[15], byte[14]) as f64 / 1000.0);
        self.sound_noise = Some(read_i16(byte[19], byte[18]) as f64 / 100.0);
        self.eTVOC = Some(read_i16(byte[21], byte[20]) as u16);
    }
}

fn read_i16(high: u8, low: u8) -> i16 {
    let high: i16 = (high as i16) << 8;
    let value: i16 = high | low as i16;
    value
}
fn read_i32(high1: u8, high2: u8, low1: u8, low2: u8) -> i32 {
    let high1: i32 = (high1 as i32) << 24;
    let high2: i32 = (high2 as i32) << 16;
    let low1: i32 = (low1 as i32) << 8;
    high1 | high2 | low1 | low2 as i32
}
#[cfg(test)]
mod tests {
    #[test]
    fn test_s16() {
        let byte = vec![
            0x52, 0x42, 0x36, 0x0, 0x1, 0x21, 0x50, 0xcf, 0x3, 0xc, 0x5b, 0x1b, 0xe7, 0x0, 0x57,
            0x48, 0xf, 0x0, 0xc9, 0x11, 0x9, 0x0, 0xd0, 0x1, 0x3b, 0x20, 0x78, 0xb, 0x0, 0x0, 0x0,
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0xa3, 0xab, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
        ];
        let r = s16(byte[9], byte[8]);
        let r = r as f64 / 100.0;
        let e = read_i32(byte[17], byte[16], byte[15], byte[14]) as f64;

        println!("{:?}", r as f64);
        println!("{:?}", e / 1000 as f64);
    }

    fn s16(high: u8, low: u8) -> i16 {
        let high: i16 = (high as i16) << 8;
        let value: i16 = high | low as i16;
        println!("0x{:04x}", high);
        println!("0x{:04x}", value);
        println!("0x{:?}", value);
        (value) as i16
    }
    fn read_i32(high1: u8, high2: u8, low1: u8, low2: u8) -> i32 {
        let high1: i32 = (high1 as i32) << 24;
        let high2: i32 = (high2 as i32) << 16;
        let low1: i32 = (low1 as i32) << 8;
        let value = high1 | high2 | low1 | low2 as i32;
        println!("0x{:04x}", value);
        value
    }
}
