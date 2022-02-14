use std::io::{Error, Write};

pub trait ToBeBytes {
    fn to_be_bytes(self) -> Box<[u8]>;
}

impl ToBeBytes for  u8 { fn to_be_bytes(self) -> Box<[u8]> { self.to_be_bytes()[..].into() } }
impl ToBeBytes for u16 { fn to_be_bytes(self) -> Box<[u8]> { self.to_be_bytes()[..].into() } }
impl ToBeBytes for u32 { fn to_be_bytes(self) -> Box<[u8]> { self.to_be_bytes()[..].into() } }
impl ToBeBytes for u64 { fn to_be_bytes(self) -> Box<[u8]> { self.to_be_bytes()[..].into() } }
impl ToBeBytes for i32 { fn to_be_bytes(self) -> Box<[u8]> { self.to_be_bytes()[..].into() } }
impl ToBeBytes for i64 { fn to_be_bytes(self) -> Box<[u8]> { self.to_be_bytes()[..].into() } }
impl ToBeBytes for f32 { fn to_be_bytes(self) -> Box<[u8]> { self.to_be_bytes()[..].into() } }
impl ToBeBytes for f64 { fn to_be_bytes(self) -> Box<[u8]> { self.to_be_bytes()[..].into() } }
impl ToBeBytes for Vec<u8> { fn to_be_bytes(self) -> Box<[u8]> { self.into_boxed_slice() } }
impl ToBeBytes for &[u8] { fn to_be_bytes(self) -> Box<[u8]> { self.into() } }

pub fn write<T: ToBeBytes>(bytes: &mut Vec<u8>, v: T) -> Result<(), Error> {
    bytes.write_all(&v.to_be_bytes())
}

pub fn write_u8(bytes: &mut Vec<u8>, v: u8) -> Result<(), Error> {
    bytes.write_all(&[v])
}

pub fn write_u16(bytes: &mut Vec<u8>, v: u16) -> Result<(), Error> {
    bytes.write_all(&v.to_be_bytes())
}

pub fn write_u32(bytes: &mut Vec<u8>, v: u32) -> Result<(), Error> {
    bytes.write_all(&v.to_be_bytes())
}

pub fn write_u64(bytes: &mut Vec<u8>, v: u64) -> Result<(), Error> {
    bytes.write_all(&v.to_be_bytes())
}

pub  fn write_i32(bytes: &mut Vec<u8>, v: i32) -> Result<(), Error> {
    bytes.write_all(&v.to_be_bytes())
}

pub  fn write_i64(bytes: &mut Vec<u8>, v: i64) -> Result<(), Error> {
    bytes.write_all(&v.to_be_bytes())
}

pub  fn write_f32(bytes: &mut Vec<u8>, v: f32) -> Result<(), Error> {
    bytes.write_all(&v.to_be_bytes())
}

pub  fn write_f64(bytes: &mut Vec<u8>, v: f64) -> Result<(), Error> {
    bytes.write_all(&v.to_be_bytes())
}
