use std::{
    collections::HashMap,
    io::{Cursor, Error, ErrorKind, Read},
};

use crate::{
    AccessFlags, Attribute, AttributeInfo, CPIndex, CodeByte, ConstantPool, ConstantPoolEntry,
    ExceptionTableEntry, Field, JavaClass, Method, ReferenceKind,
};

pub trait Deserialize {
    fn deserialize(bytes: &mut Cursor<Vec<u8>>) -> Result<Self, Error>
    where
        Self: Sized;
}

impl Deserialize for u8 {
    fn deserialize(bytes: &mut Cursor<Vec<u8>>) -> Result<u8, Error> {
        Ok(u8::from_be_bytes(Deserialize::deserialize(bytes)?))
    }
}

impl Deserialize for u16 {
    fn deserialize(bytes: &mut Cursor<Vec<u8>>) -> Result<u16, Error> {
        Ok(u16::from_be_bytes(Deserialize::deserialize(bytes)?))
    }
}

impl Deserialize for u32 {
    fn deserialize(bytes: &mut Cursor<Vec<u8>>) -> Result<u32, Error> {
        Ok(u32::from_be_bytes(Deserialize::deserialize(bytes)?))
    }
}

impl Deserialize for u64 {
    fn deserialize(bytes: &mut Cursor<Vec<u8>>) -> Result<u64, Error> {
        Ok(u64::from_be_bytes(Deserialize::deserialize(bytes)?))
    }
}

impl Deserialize for i32 {
    fn deserialize(bytes: &mut Cursor<Vec<u8>>) -> Result<i32, Error> {
        Ok(i32::from_be_bytes(Deserialize::deserialize(bytes)?))
    }
}

impl Deserialize for i64 {
    fn deserialize(bytes: &mut Cursor<Vec<u8>>) -> Result<i64, Error> {
        Ok(i64::from_be_bytes(Deserialize::deserialize(bytes)?))
    }
}

impl Deserialize for f32 {
    fn deserialize(bytes: &mut Cursor<Vec<u8>>) -> Result<f32, Error> {
        Ok(f32::from_be_bytes(Deserialize::deserialize(bytes)?))
    }
}

impl Deserialize for f64 {
    fn deserialize(bytes: &mut Cursor<Vec<u8>>) -> Result<f64, Error> {
        Ok(f64::from_be_bytes(Deserialize::deserialize(bytes)?))
    }
}

impl<const C: usize> Deserialize for [u8; C] {
    fn deserialize(bytes: &mut Cursor<Vec<u8>>) -> Result<Self, Error> {
        let mut buf = [0u8; C];
        bytes.read_exact(&mut buf)?;
        Ok(buf)
    }
}

impl<T> Deserialize for Vec<T>
where
    T: Deserialize,
{
    fn deserialize(bytes: &mut Cursor<Vec<u8>>) -> Result<Self, Error> {
        let count = u16::deserialize(bytes)? as usize;
        let mut res = Vec::with_capacity(count);

        for _ in 0..count {
            res.push(T::deserialize(bytes)?);
        }
        Ok(res)
    }
}

impl Deserialize for CPIndex {
    fn deserialize(bytes: &mut Cursor<Vec<u8>>) -> Result<Self, Error> {
        u16::deserialize(bytes)?.try_into().map_err(|_| {
            Error::new(
                ErrorKind::Other,
                "Error when trying to convert u16 to CPIndex (value is 0).",
            )
        })
    }
}

impl Deserialize for ReferenceKind {
    fn deserialize(bytes: &mut Cursor<Vec<u8>>) -> Result<Self, Error> {
        u8::deserialize(bytes)?.try_into().map_err(|_| {
            Error::new(
                ErrorKind::Other,
                "Error when trying to convert u8 to ReferenceKind",
            )
        })
    }
}

impl Deserialize for ConstantPoolEntry {
    fn deserialize(bytes: &mut Cursor<Vec<u8>>) -> Result<Self, Error> {
        let tag = u8::deserialize(bytes)?;
        match tag {
            07 => Ok(ConstantPoolEntry::Class {
                name_index: CPIndex::deserialize(bytes)?,
            }),
            09 => Ok(ConstantPoolEntry::FieldRef {
                class_index: CPIndex::deserialize(bytes)?,
                name_and_type_index: CPIndex::deserialize(bytes)?,
            }),
            10 => Ok(ConstantPoolEntry::MethodRef {
                class_index: CPIndex::deserialize(bytes)?,
                name_and_type_index: CPIndex::deserialize(bytes)?,
            }),
            11 => Ok(ConstantPoolEntry::InterfaceMethodRef {
                class_index: CPIndex::deserialize(bytes)?,
                name_and_type_index: CPIndex::deserialize(bytes)?,
            }),
            08 => Ok(ConstantPoolEntry::String {
                string_index: CPIndex::deserialize(bytes)?,
            }),
            03 => Ok(ConstantPoolEntry::Integer(i32::deserialize(bytes)?)),
            04 => Ok(ConstantPoolEntry::Float(f32::deserialize(bytes)?)),
            05 => Ok(ConstantPoolEntry::Long(i64::deserialize(bytes)?)),
            06 => Ok(ConstantPoolEntry::Double(f64::deserialize(bytes)?)),
            12 => Ok(ConstantPoolEntry::NameAndType {
                name_index: CPIndex::deserialize(bytes)?,
                descriptor_index: CPIndex::deserialize(bytes)?,
            }),
            01 => {
                let len = u16::deserialize(bytes)?;
                let mut buf = vec![0u8; len as usize];
                bytes.read_exact(buf.as_mut_slice())?;
                Ok(ConstantPoolEntry::Utf8(
                    String::from_utf8_lossy(&buf).into(),
                ))
            }
            15 => Ok(ConstantPoolEntry::MethodHandle {
                reference_kind: ReferenceKind::deserialize(bytes)?,
                reference_index: CPIndex::deserialize(bytes)?,
            }),
            16 => Ok(ConstantPoolEntry::MethodType {
                descriptor_index: CPIndex::deserialize(bytes)?,
            }),
            18 => Ok(ConstantPoolEntry::InvokeDynamic {
                bootstrap_method_attr_index: u16::deserialize(bytes)?,
                name_and_type_index: CPIndex::deserialize(bytes)?,
            }),
            _ => Err(Error::new(
                ErrorKind::Other,
                "Unkown tag on ConstantPoolEntry",
            )),
        }
    }
}

impl Deserialize for ConstantPool {
    fn deserialize(bytes: &mut Cursor<Vec<u8>>) -> Result<ConstantPool, Error> {
        let count = u16::deserialize(bytes)?;
        let mut index = 1u16; // indices starts at 1
        let mut map = HashMap::new();

        while index < count {
            let entry = ConstantPoolEntry::deserialize(bytes)?;
            let size = entry.size();

            map.insert(index.try_into().unwrap(), entry);
            index += size;
        }

        Ok(Self { inner: map })
    }
}

impl Deserialize for AccessFlags {
    fn deserialize(bytes: &mut Cursor<Vec<u8>>) -> Result<Self, Error> {
        AccessFlags::from_bits(u16::deserialize(bytes)?).ok_or(Error::new(
            ErrorKind::Other,
            "Error when trying to convert u16 to AccessFlags",
        ))
    }
}

impl Deserialize for Field {
    fn deserialize(bytes: &mut Cursor<Vec<u8>>) -> Result<Self, Error> {
        let access_flags = AccessFlags::deserialize(bytes)?;
        let name_index = CPIndex::deserialize(bytes)?;
        let descriptor_index = CPIndex::deserialize(bytes)?;
        let attributes = Vec::<Attribute>::deserialize(bytes)?;

        Ok(Self {
            access_flags,
            name_index,
            descriptor_index,
            attributes,
        })
    }
}

impl Deserialize for Method {
    fn deserialize(bytes: &mut Cursor<Vec<u8>>) -> Result<Self, Error> {
        let access_flags = AccessFlags::deserialize(bytes)?;
        let name_index = CPIndex::deserialize(bytes)?;
        let descriptor_index = CPIndex::deserialize(bytes)?;
        let attributes = Vec::<Attribute>::deserialize(bytes)?;

        Ok(Self {
            access_flags,
            name_index,
            descriptor_index,
            attributes,
        })
    }
}

impl Deserialize for ExceptionTableEntry {
    fn deserialize(bytes: &mut Cursor<Vec<u8>>) -> Result<Self, Error> {
        Ok(Self {
            start: u16::deserialize(bytes)?,
            end: u16::deserialize(bytes)?,
            handler: u16::deserialize(bytes)?,
            catch_type: CPIndex::deserialize(bytes)?,
        })
    }
}

impl Deserialize for CodeByte {
    fn deserialize(bytes: &mut Cursor<Vec<u8>>) -> Result<Self, Error> {
        Ok(Self(u8::deserialize(bytes)?))
    }
}

impl Deserialize for AttributeInfo {
    fn deserialize(bytes: &mut Cursor<Vec<u8>>) -> Result<Self, Error> {
        let size = u32::deserialize(bytes)?;
        let mut buf = vec![0u8; size as usize];
        bytes.read_exact(buf.as_mut_slice())?;
        Ok(AttributeInfo::Any(buf))
    }
}

impl Deserialize for Attribute {
    fn deserialize(bytes: &mut Cursor<Vec<u8>>) -> Result<Self, Error> {
        let name_index = CPIndex::deserialize(bytes)?;
        let info = AttributeInfo::deserialize(bytes)?;

        Ok(Self { name_index, info })
    }
}

impl Deserialize for JavaClass {
    fn deserialize(bytes: &mut Cursor<Vec<u8>>) -> Result<Self, Error> {
        let magic_bytes = u32::deserialize(bytes)?;
        let minor_version = u16::deserialize(bytes)?;
        let major_version = u16::deserialize(bytes)?;
        let constant_pool = ConstantPool::deserialize(bytes)?;
        let access_flags = AccessFlags::deserialize(bytes)?;
        let this_class = CPIndex::deserialize(bytes)?;
        let super_class = CPIndex::deserialize(bytes).ok(); // optional
        let interfaces = Vec::<CPIndex>::deserialize(bytes)?;
        let mut fields = Vec::<Field>::deserialize(bytes)?;
        let mut methods = Vec::<Method>::deserialize(bytes)?;
        let mut attributes = Vec::<Attribute>::deserialize(bytes)?;

        // resolve the attributes
        for f in fields.iter_mut() {
            for a in f.attributes.iter_mut() {
                let _ = a.resolve(&constant_pool);
            }
        }
        for m in methods.iter_mut() {
            for a in m.attributes.iter_mut() {
                let _ = a.resolve(&constant_pool);
            }
        }
        for a in attributes.iter_mut() {
            let _ = a.resolve(&constant_pool);
        }

        Ok(Self {
            magic_bytes,
            minor_version,
            major_version,
            constant_pool,
            access_flags,
            this_class,
            super_class,
            interfaces,
            fields,
            methods,
            attributes,
        })
    }
}
