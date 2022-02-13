use std::io::{Cursor, Read, Error};

pub fn read_u8(bytes: &mut Cursor<Vec<u8>>) -> Result<u8, Error> {
    let mut buf = [0u8];
    bytes.read_exact(&mut buf)?;
    Ok(u8::from_be(buf[0]))
}

pub fn read_u16(bytes: &mut Cursor<Vec<u8>>) -> Result<u16, Error> {
    let mut buf = [0u8,0u8];
    bytes.read_exact(&mut buf)?;
    Ok(u16::from_be_bytes(buf))
}

pub fn read_u32(bytes: &mut Cursor<Vec<u8>>) -> Result<u32, Error> {
    let mut buf = [0u8,0u8,0u8,0u8];
    bytes.read_exact(&mut buf)?;
    Ok(u32::from_be_bytes(buf))
}

pub fn read_u64(bytes: &mut Cursor<Vec<u8>>) -> Result<u64, Error> {
    let mut buf = [0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8];
    bytes.read_exact(&mut buf)?;
    Ok(u64::from_be_bytes(buf))
}

pub  fn read_i32(bytes: &mut Cursor<Vec<u8>>) -> Result<i32, Error> {
    let mut buf = [0u8,0u8,0u8,0u8];
    bytes.read_exact(&mut buf)?;
    Ok(i32::from_be_bytes(buf))
}

pub  fn read_i64(bytes: &mut Cursor<Vec<u8>>) -> Result<i64, Error> {
    let mut buf = [0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8];
    bytes.read_exact(&mut buf)?;
    Ok(i64::from_be_bytes(buf))
}

pub  fn read_f32(bytes: &mut Cursor<Vec<u8>>) -> Result<f32, Error> {
    let mut buf = [0u8,0u8,0u8,0u8];
    bytes.read_exact(&mut buf)?;
    Ok(f32::from_be_bytes(buf))
}

pub  fn read_f64(bytes: &mut Cursor<Vec<u8>>) -> Result<f64, Error> {
    let mut buf = [0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8];
    bytes.read_exact(&mut buf)?;
    Ok(f64::from_be_bytes(buf))
}

pub fn read_bytes(bytes: &mut Cursor<Vec<u8>>, size: usize) -> Result<Vec<u8>, Error> {
    let mut buf = vec![0u8;size];
    bytes.read_exact(buf.as_mut_slice())?;
    Ok(buf)
}
