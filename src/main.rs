extern crate rppal;

use futures::executor;
use std::thread;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use rppal::i2c::I2c;
use rppal::gpio::Gpio;
fn main() {
    println!("Hello, world!");
    let mut handles= Vec::new();
    let data =Arc::new(Mutex::new(vec![1;10]));

    for x in 0..10{
        let data_ref= data.clone();
        handles.push(thread::spawn(move || {
            //
            let mut data = data_ref.lock().unwrap();
            data[x] += 1;
        }))
    }

    for handle in handles{
        let _ = handle.join();
    }
    dbg!(&data);
    let mut gpio = Gpio::new().expect("failed to get gpio!");
    dbg!(&gpio);
}

async fn add(left:i32,right:i32)->i32{
    left+right
}
