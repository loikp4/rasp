use rumqtt::{MqttClient, MqttOptions, QoS, ReconnectOptions, mqttoptions};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::{read, File};
use std::io::prelude::*;
use std::path::Path;

#[derive(Clone)]
#[allow(non_snake_case)]
pub struct Mqtt {
    AWS_Endpoint: String,
    mqttclient: MqttClient
}

impl Mqtt {
    pub fn new() -> Mqtt {
        let rootCApath = Path::new("/home/master/rasp/cert/root-CA.crt");
        let privatekeypath = Path::new("/home/master/rasp/cert/raspidevice.private.key");
        let certpath = Path::new("/home/master/rasp/cert/raspidevice.cert.pem");
        let mqttoptions = MqttOptions::new(
            "sdk-nodejs-7c641eb5-0c2e-4e2e-9bb5-c8920275ae7c",
            "a3tzmb0oyi31tk-ats.iot.ap-northeast-1.amazonaws.com",
            8883,
        )
        .set_ca(read(&rootCApath).expect("cannot open rootCA"))
        .set_client_auth(read(&certpath).unwrap(), read(&privatekeypath).unwrap())
        .set_keep_alive(10)
        .set_connection_timeout(5)
        .set_reconnect_opts(ReconnectOptions::Always(5));

        let (mut mqttclient, notifications) = MqttClient::start(mqttoptions).expect("connexct error");

        let r = match mqttclient.subscribe("topic_1", QoS::AtMostOnce){
            Ok(f) => println!("subscribe"),
            Err(e) => println!("client error = {:?}",e),
        };
        dbg!(&notifications);
        for notification in notifications{
            println!("{:?}",notification);

        }
        //rootcaファイルをオープン
        /*let rootca = match File::open(&rootCApath) {
                    Ok(mut f) => f.read(&mut s[0]),
                    Err(e) => panic!("cannot open {}", e.to_string()),
                };
                let _ = match File::open(&privatekeypath) {
                    Ok(mut f) => f.read(&mut s[1]),
                    Err(e) => panic!("cannot open {}", e.to_string()),
                };
                let _ = match File::open(&certpath) {
                    Ok(mut f) => f.read(&mut s[2]),
                    Err(e) => panic!("cannot open {}", e.to_string()),
                };
                let mqttoptions = MqttOptions::new(
                    "0",
                    "a3tzmb0oyi31tk-ats.iot.ap-northeast-1.amazonaws.com",
                    8883,
                );

        */
        Mqtt {
            AWS_Endpoint: "a3tzmb0oyi31tk-ats.iot.ap-northeast-1.amazonaws.com".to_string(),
            mqttclient
        }
    }

    pub fn connect(mut self) {
        
    }

    pub fn publish(&mut self,payload:&str){
       match self.mqttclient.publish("topic_1", QoS::AtLeastOnce, false, payload){
           Ok(f) => println!("published!"),
           Err(e) => println!("client error = {:?}",e),
       };

    }
}
