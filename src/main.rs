extern crate rppal;

use rppal::gpio::Gpio;
use rppal::i2c::I2c;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tokio;

const ADDR_I2C: u16 = 0x76;
const REG_CTRL_HUM: u8 = 0xF2;
const REG_ADC_VALUE: u8 = 0xF7;
const ACC_I2C_ADDRESS: u16 = 0x19;// BMX055　加速度センサのI2Cアドレス  
const GYRO_I2C_ADDRESS: u16 = 0x69;// BMX055　ジャイロセンサのI2Cアドレス
const MAG_I2C_ADDRESS: u16 = 0x13;// BMX055　磁気センサのI2Cアドレス

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    //let p = acquire().map(|data| println!("{}",data)).expect("failed to get i2c");
    loop {
        let accz = bmx055_acc().unwrap();
        dbg!(&accz);
        let gyro = BMX055_gyro();
        dbg!(&gyro);
        let mag= BMX055_Mag();
        dbg!(&mag);
        thread::sleep(Duration::from_secs_f32(0.5));
    }
    let mut gpio = Gpio::new().expect("failed to get gpio!");
    dbg!(&gpio);
}
fn bmx055_acc() -> Result<(i16, i16, i16), rppal::i2c::Error> {
    let mut i2c = I2c::new()?;
    i2c.set_slave_address(ACC_I2C_ADDRESS)?;
    let mut data: Vec<u16> = vec![0; 6];
    for i in 0..6 {
        data[i] = i2c.smbus_read_byte((2 + i) as u8)? as u16;
    }
    let mut acc_x: u16 = ((data[1] * 256) + (data[0] & 0xF0)) / 16;
    if acc_x > 2047 {
        acc_x -= 4096;
    }
    let mut acc_y: u16 = ((data[3] * 256) + (data[2] & 0xF0)) / 16;
    if acc_y > 2047 {
        acc_y -= 4096;
    }
    let mut acc_z: u16 = ((data[5] * 256) + (data[4] & 0xF0)) / 16;
    if acc_z > 2047 {
        acc_z -= 4096;
    }

    Ok((acc_x as i16, acc_y as i16, acc_z as i16))
}

fn BMX055_gyro() -> Result<(i16, i16, i16), rppal::i2c::Error> {
    let mut i2c = I2c::new()?;
    i2c.set_slave_address(GYRO_I2C_ADDRESS)?;
    let mut data: Vec<u32> = vec![0; 6];
    for i in 0..6 {
        data[i] = i2c.smbus_read_byte((2 + i) as u8)? as u32;
    }
    let mut gyro_x: u32 = (data[1] * 256) + data[0];
    if gyro_x > 32767 {
        gyro_x -= 65536;
    }
    let mut gyro_y: u32 = (data[3] * 256) + data[2];
    if gyro_x > 32767 {
        gyro_x -= 65536;
    }
    let mut gyro_z: u32 = (data[5] * 256) + data[4];
    if gyro_x > 32767 {
        gyro_x -= 65536;
    }
    Ok((gyro_x as i16, gyro_y as i16, gyro_z as i16))
}
fn BMX055_Mag() -> Result<(i16, i16, i16), rppal::i2c::Error> {
    let mut i2c = I2c::new()?;
    i2c.set_slave_address(MAG_I2C_ADDRESS)?;
    let mut data: Vec<u16> = vec![0; 8];
    for i in 0..8 {
        data[i] = i2c.smbus_read_byte((0x42 + i) as u8)? as u16;
    }

    let mut xMag = (data[1] << 8) | (data[0] >> 3);
    if xMag > 4095 {
        xMag -= 8192;
    }
    let mut yMag = (data[3] << 8) | (data[2] >> 3);
    if yMag > 4095 {
        yMag -= 8192;
    }
    let mut zMag = (data[5] << 8) | (data[4] >> 3);
    if zMag > 16383 {
        zMag -= 32768;
    }
    Ok((xMag as i16, yMag as i16, zMag as i16))
}
fn acquire() -> Result<u8, rppal::i2c::Error> {
    let mut i2c = I2c::new()?;
    i2c.set_slave_address(ADDR_I2C)?;
    i2c.smbus_write_byte(REG_CTRL_HUM, 1u8)?;
    let data: u8 = i2c.smbus_read_byte(REG_ADC_VALUE)?;
    Ok(data)
}
