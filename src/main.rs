extern crate serialport;

mod usb_2jcie;

use core::time::Duration;
use rumqtt::{mqttoptions, MqttClient, MqttOptions, QoS, ReconnectOptions};

use serialport::SerialPort;
use std::fs::read;
use std::io::prelude::*;
use std::path::Path;

use std::thread;
// use std::time::Duration;
use tokio;

#[tokio::main]
async fn main() {
    /*let ports = serialport::available_ports().expect("No ports found");
    for p in ports{
        dbg!(&p);
    }*/
    let mut data = usb_2jcie::usb_2jcie::new();
    let rootCApath = Path::new(r"./cert/root-CA.crt");
    let privatekeypath = Path::new(r"./cert/raspidevice.private.key");
    let certpath = Path::new(r"./cert/raspidevice.cert.pem");
    let mqttoptions: MqttOptions = MqttOptions::new(
        "sdk-nodejs-7c6-rasp4",
        "a3tzmb0oyi31tk-ats.iot.ap-northeast-1.amazonaws.com",
        8883,
    )
    .set_ca(read(&rootCApath).expect("cannot open rootCA"))
    .set_client_auth(
        read(&certpath).expect("cannot open cert.pem"),
        read(&privatekeypath).expect("cannot open privatekey"),
    )
    .set_keep_alive(10)
    .set_connection_timeout(5)
    .set_reconnect_opts(ReconnectOptions::Always(5));
    let (mut mqttclient, notifications) = 
    MqttClient::start(mqttoptions).expect("connect error");

    let _r = match mqttclient.subscribe("topic_1", QoS::AtMostOnce) {
        Ok(_f) => println!("subscribe"),
        Err(e) => println!("client error = {:?}", e),
    };

    let mut serial = serialport::new("COM3", 115200)
        .timeout(Duration::from_millis(100))
        .open()
        .expect("failed to open port");

    thread::spawn(move || {
        loop {
            let res = getlongdata(&mut serial);
            match res {
                Ok(longdata) => {
                    let index = getindex(&longdata).unwrap_or_else(|| 0);
                    data.parse(&longdata[index..]);
                    //json文字列を作る
                    let json = serde_json::to_string(&data).unwrap();
                    thread::sleep(Duration::from_millis(100));
                    match mqttclient.publish("topic_1", QoS::AtLeastOnce, false, json.as_str()) {
                        Ok(_f) => println!("data = {:?}", json),
                        Err(e) => println!("client error = {:?}", e),
                    }
                }
                Err(_) => todo!(),
            }
            thread::sleep(Duration::from_secs(12));
        }
    });

    for notification in notifications {
        //println!("{:?}", notification)
    }
}



fn show(byte: &Vec<u8>) {
    for i in byte {
        print!("0x{:x},", i);
    }
    println!();
}

fn crc16(bytes: &[u8], from_index: usize, to_index: usize) -> (u8, u8) {
    let mut crc: u16 = 0xFFFF;
    for i in from_index..to_index {
        let b = bytes[i] as u16;
        crc = crc ^ b;
        for j in 0..8 {
            let lsb = crc & 1;
            crc = crc >> 1 & 0x7FFF;
            if lsb == 1 {
                crc = crc ^ 0xA001;
            }
        }
    }
    let crc_l = (crc & 0x00FF) as u8;
    let crc_h = (crc >> 8) as u8;
    return (crc_l, crc_h);
}

fn getlongdata(ser: &mut Box<dyn SerialPort>) -> Result<Vec<u8>, serialport::Error> {
    let mut command = vec![
        0x52u8, 0x42, 0x0a, 0x00, 0x02, 0x11, 0x51, 0x01, 0x00, 255, 255, 0,
    ]; // LED ON
    let (crc_l, crc_h) = crc16(&command, 0, command.len());
    command.push(crc_l);
    command.push(crc_h);

    let mut command2 = vec![0x52u8, 0x42, 0x05, 0x00, 0x01, 0x21, 0x50]; //get latest data long
    let (crc_l, crc_h) = crc16(&command2, 0, command2.len());
    command2.push(crc_l);
    command2.push(crc_h);

    ser.write_all(&command2)?; //send for getting latest data long
    thread::sleep(Duration::from_millis(100));
    let mut res: Vec<u8> = vec![0; 90];
    ser.read_exact(res.as_mut())?;

    show(&res);
    Ok(res)
}

//usbからLongdataが取得できているかどうかを探る イテレータの関数でなんとかできないか
fn getindex(longdata: &[u8]) -> Option<usize> {
    let checking = [0x52u8, 0x42, 0x36, 0x0, 0x1, 0x21, 0x50];
    for (i, num) in longdata.iter().enumerate() {
        for j in checking.iter().enumerate() {
            if longdata[i + j.0] == checking[j.0] {
            } else {
                break;
            }

            if j.0 == checking.len() - 1 {
                return Some(i);
            }
        }
    }
    None
}
#[cfg(test)]
mod tests {
    use std::vec;

    use futures::executor::BlockingStream;
    use serde::Serialize;

    use super::*;

    #[test]
    fn test_crc16() {
        let mut command = vec![0x52, 0x42, 0x05, 0x00, 0x01, 0x21, 0x50];
        let (crc1, crc2) = crc16(&command, 0, command.len());
        println!("crc16 = 0x{:x}, 0x{:x}", crc1, crc2);
    }
    fn read_i16(high: u8, low: u8) -> u16 {
        let higher: u16 = (high as u16) << 8;
        let value: u16 = higher + low as u16;
        value
    }
    #[test]
    fn tests16() {
        let header2 = vec![0x52u16, 0x42, 0x05, 0x03, 0x55];
        let a = read_i16(0x51, 0x7b);
        println!("0x{:}", a as f64 / 100.0);
    }

    #[test]
    fn stru() {
        let new = vec![
            0x01, 0x52, 0x42, 0x36, 0x0, 0x1, 0x21, 0x50, 0x6a, 0x1, 0xb, 0x97, 0x15, 0x72, 0x0,
            0x84, 0x24, 0xf, 0x0, 0xb6, 0x20, 0x17, 0x0, 0x29, 0x2, 0xee, 0x1d, 0xac, 0x9, 0x1,
            0x14, 0x0, 0x34, 0x3, 0xcc, 0xb, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x5c, 0xab, 0x0, 0x0, 0x0, 0x0, 0x0,
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
        ];
        let new = vec![
            0x52u8, 0x42, 0xa, 0x0, 0x2, 0x11, 0x51, 0x1, 0x0, 0xff, 0xff, 0x0, 0xe2, 0x5, 0x52,
            0x42, 0x36, 0x0, 0x1, 0x21, 0x50, 0x5c, 0x23, 0xc, 0xf4, 0x19, 0xeb, 0x0, 0x60, 0x43,
            0xf, 0x0, 0xef, 0x11, 0x37, 0x0, 0xfb, 0x2, 0x30, 0x20, 0x58, 0xb, 0x0, 0x0, 0x0, 0x0,
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0xe0, 0x1d, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
            0x0, 0x0, 0x0,
        ];

        let pp = vec![
            0x52u8, 0x42, 0xa, 0x0, 0x2, 0x11, 0x51, 0x1, 0x0, 0xff, 0xff, 0x0, 0xe2, 0x5, 0x52,
            0x42, 0x36, 0x0, 0x1, 0x21, 0x50, 0xe9, 0x95, 0xb, 0x95, 0x1b, 0xf1, 0x0, 0xb9, 0x48,
            0xf, 0x0, 0xb6, 0x28, 0x15, 0x0, 0x1e, 0x2, 0x9e, 0x1f, 0x12, 0xb, 0x1, 0xdf, 0x0,
            0xc4, 0x22, 0xd0, 0x13, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x7f, 0x96, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
        ];
        // let a = new.iter().scan(1,|initial_state, &f|{
        //Some()
        // });
        //index付きのループ enumerate()
        //usbからLongdataが取得できているかどうかを探る イテレータの関数でなんとかできないか
        let index = getindex(&pp).unwrap();
        let index2 = getindex2(&pp);
        let lter = pp.iter().enumerate().skip(index);

        dbg!(index);
        dbg!(index2);
        let mut sam = usb_2jcie::usb_2jcie::new();
        sam.parse(&pp[index..]);
        println!("{:?}", &sam);
    }

    //usbからLongdataが取得できているかどうかを探る
    fn getindex(d: &Vec<u8>) -> Option<usize> {
        let checking = [0x52u8, 0x42, 0x36, 0x0, 0x1, 0x21, 0x50];
        for (i, num) in d.iter().enumerate() {
            for j in checking.iter().enumerate() {
                if d[i + j.0] == checking[j.0] {
                    println!("{:x}", j.1);
                } else {
                    break;
                }

                //配列checkingが含まれていたらreturn
                if j.0 == checking.len() - 1 {
                    return Some(i);
                }
            }
        }
        None
    }

    fn getindex2(d: &Vec<u8>) -> Option<usize> {
        let checking = [0x52u8, 0x42, 0x36, 0x0, 0x1, 0x21, 0x50];
        let a = d.iter().enumerate().map(|(i, &num)| {
            let b = checking.iter();
            print!("{:?}", i);
        });
        None
    }
    #[test]
    fn mqtt2() {
        use rumqtt::{mqttoptions, MqttClient, MqttOptions, QoS, ReconnectOptions};
        use serde::{Deserialize, Serialize};
        use std::error::Error;
        use std::fs::{read, File};
        use std::path::Path;
        let rootCApath = Path::new(r"..\cert\root-CA.crt");
        let privatekeypath = Path::new("../cert/raspidevice.private.key");
        let certpath = Path::new("/home/master/rasp/cert/raspidevice.cert.pem");
        let mqttoptions = MqttOptions::new(
            "sdk-nodejs-7c6",
            "a3tzmb0oyi31tk-ats.iot.ap-northeast-1.amazonaws.com",
            8883,
        )
        .set_ca(read(&rootCApath).expect("cannot open rootCA"))
        .set_client_auth(read(&certpath).unwrap(), read(&privatekeypath).unwrap())
        .set_keep_alive(10)
        .set_connection_timeout(5)
        .set_reconnect_opts(ReconnectOptions::Always(5));

        let (mut mqttclient, notifications) =
            MqttClient::start(mqttoptions).expect("connexct error");

        let r = match mqttclient.subscribe("topic_1", QoS::AtMostOnce) {
            Ok(f) => println!("subscribe"),
            Err(e) => println!("client error = {:?}", e),
        };
        let sleep_time = Duration::from_secs(10);

        thread::spawn(move || {
            loop {
                let d = vec![
                    0x52, 0x42, 0x36, 0x0, 0x1, 0x21, 0x50, 0x6a, 0x1, 0xb, 0x97, 0x15, 0x72, 0x0,
                    0x84, 0x24, 0xf, 0x0, 0xb6, 0x20, 0x17, 0x0, 0x29, 0x2, 0xee, 0x1d, 0xac, 0x9,
                    0x1, 0x14, 0x0, 0x34, 0x3, 0xcc, 0xb, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
                    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x5c, 0xab,
                    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
                    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
                ];
                let mut sam = usb_2jcie::usb_2jcie::new();
                sam.parse(&d);
                //json文字列を作る
                let json = serde_json::to_string(&sam).unwrap();
                thread::sleep(sleep_time);
                match mqttclient.publish("topic_1", QoS::AtLeastOnce, false, json.as_str()) {
                    Ok(f) => println!("published!"),
                    Err(e) => println!("client error = {:?}", e),
                };
            }
        });
        for notification in notifications {
            println!("{:?}", notification)
        }
    }

    #[test]
    fn http() {
        use reqwest;
        use usb_2jcie;
        let d = vec![
            0x52, 0x42, 0x36, 0x0, 0x1, 0x21, 0x50, 0x6a, 0x1, 0xb, 0x97, 0x15, 0x72, 0x0, 0x84,
            0x24, 0xf, 0x0, 0xb6, 0x20, 0x17, 0x0, 0x29, 0x2, 0xee, 0x1d, 0xac, 0x9, 0x1, 0x14,
            0x0, 0x34, 0x3, 0xcc, 0xb, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x5c, 0xab, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
            0x0, 0x0, 0x0, 0x0, 0x0,
        ];

        let mut sam = usb_2jcie::usb_2jcie::new();
        sam.parse(&d);
        //json文字列を作る
        let json = serde_json::to_string(&sam).unwrap();
        dbg!(&json);
        //let body = reqwest::blocking::get("http://192.168.1.4:1880/getting").unwrap().text();
        let client = reqwest::blocking::Client::new();
        let body2 = client
            .post("http://192.168.1.4:1880/post")
            .body(json)
            .send()
            .unwrap();
        dbg!(body2);
    }
    #[test]
    fn getdata() {
        /*let ports = serialport::available_ports().expect("No ports found");
        for p in ports {
            dbg!(&p);
        }*/

        let mut command = vec![
            0x52u8, 0x42, 0x0a, 0x00, 0x02, 0x11, 0x51, 0x01, 0x00, 255, 255, 0,
        ]; // LED ON
        let (crc_l, crc_h) = crc16(&command, 0, command.len());
        command.push(crc_l);
        command.push(crc_h);

        let mut command2 = vec![0x52u8, 0x42, 0x05, 0x00, 0x01, 0x21, 0x50]; //get latest data long
        let (crc_l, crc_h) = crc16(&command2, 0, command2.len());
        command2.push(crc_l);
        command2.push(crc_h);
        let mut serial = serialport::new("/dev/ttyUSB0", 115200)
            .timeout(Duration::from_millis(100))
            .open()
            .expect("failed to open port");

        serial.write_all(&command).unwrap();
        thread::sleep(Duration::from_millis(100));
        serial.write_all(&command2).unwrap();
        thread::sleep(Duration::from_millis(100));
        let mut res: Vec<u8> = vec![0; 70];
        serial.read(res.as_mut()).unwrap();

        show(&res);
    }
}
