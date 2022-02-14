use crate::{
    AccessFlags, Attribute, AttributeInfo, CPIndex, CodeByte, ConstantPool, ConstantPoolEntry,
    ExceptionTableEntry, Field, JavaClass, Method, ReferenceKind,
};
use std::io::{Error, Write};

pub trait Serialize {
    fn serialize(&self, bytes: &mut Vec<u8>) -> Result<(), Error>;
}
impl Serialize for u8 {
    fn serialize(&self, bytes: &mut Vec<u8>) -> Result<(), Error> {
        bytes.write_all(&[*self])
    }
}
impl Serialize for u16 {
    fn serialize(&self, bytes: &mut Vec<u8>) -> Result<(), Error> {
        bytes.write_all(&self.to_be_bytes())
    }
}
impl Serialize for u32 {
    fn serialize(&self, bytes: &mut Vec<u8>) -> Result<(), Error> {
        bytes.write_all(&self.to_be_bytes())
    }
}
impl Serialize for u64 {
    fn serialize(&self, bytes: &mut Vec<u8>) -> Result<(), Error> {
        bytes.write_all(&self.to_be_bytes())
    }
}
impl Serialize for i32 {
    fn serialize(&self, bytes: &mut Vec<u8>) -> Result<(), Error> {
        bytes.write_all(&self.to_be_bytes())
    }
}
impl Serialize for i64 {
    fn serialize(&self, bytes: &mut Vec<u8>) -> Result<(), Error> {
        bytes.write_all(&self.to_be_bytes())
    }
}
impl Serialize for f32 {
    fn serialize(&self, bytes: &mut Vec<u8>) -> Result<(), Error> {
        bytes.write_all(&self.to_be_bytes())
    }
}
impl Serialize for f64 {
    fn serialize(&self, bytes: &mut Vec<u8>) -> Result<(), Error> {
        bytes.write_all(&self.to_be_bytes())
    }
}
impl Serialize for String {
    fn serialize(&self, bytes: &mut Vec<u8>) -> Result<(), Error> {
        bytes.write_all(&self.as_bytes()[..])
    }
}
impl<T> Serialize for Vec<T>
where
    T: Serialize,
{
    fn serialize(&self, bytes: &mut Vec<u8>) -> Result<(), Error> {
        (self.len() as u16).serialize(bytes)?;
        for i in self.iter() {
            i.serialize(bytes)?;
        }
        Ok(())
    }
}

impl Serialize for CPIndex {
    fn serialize(&self, bytes: &mut Vec<u8>) -> Result<(), Error> {
        self.0.serialize(bytes)
    }
}
impl Serialize for Option<CPIndex> {
    fn serialize(&self, bytes: &mut Vec<u8>) -> Result<(), Error> {
        match self {
            Some(cpi) => cpi.serialize(bytes),
            None => CPIndex::none().serialize(bytes),
        }
    }
}
impl Serialize for ReferenceKind {
    fn serialize(&self, bytes: &mut Vec<u8>) -> Result<(), Error> {
        (*self as u8).serialize(bytes)
    }
}
impl Serialize for ConstantPoolEntry {
    fn serialize(&self, bytes: &mut Vec<u8>) -> Result<(), Error> {
        match self {
            ConstantPoolEntry::Class { name_index } => {
                (7u8).serialize(bytes)?;
                name_index.serialize(bytes)
            }
            ConstantPoolEntry::FieldRef {
                class_index,
                name_and_type_index,
            } => {
                (9u8).serialize(bytes)?;
                class_index.serialize(bytes)?;
                name_and_type_index.serialize(bytes)
            }
            ConstantPoolEntry::MethodRef {
                class_index,
                name_and_type_index,
            } => {
                (10u8).serialize(bytes)?;
                class_index.serialize(bytes)?;
                name_and_type_index.serialize(bytes)
            }
            ConstantPoolEntry::InterfaceMethodRef {
                class_index,
                name_and_type_index,
            } => {
                (11u8).serialize(bytes)?;
                class_index.serialize(bytes)?;
                name_and_type_index.serialize(bytes)
            }
            ConstantPoolEntry::String { string_index } => {
                (8u8).serialize(bytes)?;
                string_index.serialize(bytes)
            }
            ConstantPoolEntry::Integer(n) => {
                (3u8).serialize(bytes)?;
                n.serialize(bytes)
            }
            ConstantPoolEntry::Float(n) => {
                (4u8).serialize(bytes)?;
                n.serialize(bytes)
            }
            ConstantPoolEntry::Long(n) => {
                (5u8).serialize(bytes)?;
                n.serialize(bytes)
            }
            ConstantPoolEntry::Double(n) => {
                (6u8).serialize(bytes)?;
                n.serialize(bytes)
            }
            ConstantPoolEntry::NameAndType {
                name_index,
                descriptor_index,
            } => {
                (12u8).serialize(bytes)?;
                name_index.serialize(bytes)?;
                descriptor_index.serialize(bytes)
            }
            ConstantPoolEntry::Utf8(s) => {
                (1u8).serialize(bytes)?;
                (s.len() as u16).serialize(bytes)?;
                s.serialize(bytes)
            }
            ConstantPoolEntry::MethodHandle {
                reference_kind,
                reference_index,
            } => {
                (15u8).serialize(bytes)?;
                reference_kind.serialize(bytes)?;
                reference_index.serialize(bytes)
            }
            ConstantPoolEntry::MethodType { descriptor_index } => {
                (16u8).serialize(bytes)?;
                descriptor_index.serialize(bytes)
            }
            ConstantPoolEntry::InvokeDynamic {
                bootstrap_method_attr_index,
                name_and_type_index,
            } => {
                (18u8).serialize(bytes)?;
                bootstrap_method_attr_index.serialize(bytes)?;
                name_and_type_index.serialize(bytes)
            }
        }
    }
}

impl Serialize for ConstantPool {
    fn serialize(&self, bytes: &mut Vec<u8>) -> Result<(), Error> {
        self.size().serialize(bytes)?;
        // all Entries sort by index
        let mut entries = self.iter().collect::<Vec<(&CPIndex, &ConstantPoolEntry)>>();
        entries.sort_by(|(a, _), (b, _)| a.cmp(&b));

        for (_, v) in entries.iter() {
            v.serialize(bytes)?;
        }

        Ok(())
    }
}

impl Serialize for AccessFlags {
    fn serialize(&self, bytes: &mut Vec<u8>) -> Result<(), Error> {
        self.bits.serialize(bytes)
    }
}

impl Serialize for Field {
    fn serialize(&self, bytes: &mut Vec<u8>) -> Result<(), Error> {
        self.access_flags.serialize(bytes)?;
        self.name_index.serialize(bytes)?;
        self.descriptor_index.serialize(bytes)?;
        self.attributes.serialize(bytes)
    }
}

impl Serialize for Method {
    fn serialize(&self, bytes: &mut Vec<u8>) -> Result<(), Error> {
        self.access_flags.serialize(bytes)?;
        self.name_index.serialize(bytes)?;
        self.descriptor_index.serialize(bytes)?;
        self.attributes.serialize(bytes)
    }
}

impl Serialize for ExceptionTableEntry {
    fn serialize(&self, bytes: &mut Vec<u8>) -> Result<(), Error> {
        self.start.serialize(bytes)?;
        self.end.serialize(bytes)?;
        self.handler.serialize(bytes)?;
        self.catch_type.serialize(bytes)
    }
}

impl Serialize for CodeByte {
    fn serialize(&self, bytes: &mut Vec<u8>) -> Result<(), Error> {
        self.0.serialize(bytes)
    }
}

impl Serialize for AttributeInfo {
    fn serialize(&self, bytes: &mut Vec<u8>) -> Result<(), Error> {
        match self {
            AttributeInfo::Any(b) => bytes.write_all(&b[..]),
            AttributeInfo::Code {
                max_stack,
                max_locals,
                code,
                exception_table,
                attributes,
            } => {
                max_stack.serialize(bytes)?;
                max_locals.serialize(bytes)?;
                (code.len() as u32).serialize(bytes)?;
                for i in code.iter() {
                    i.serialize(bytes)?;
                }
                exception_table.serialize(bytes)?;
                attributes.serialize(bytes)
            }
            AttributeInfo::Exceptions {
                exception_index_table,
            } => exception_index_table.serialize(bytes),
            AttributeInfo::ConstantValue { index } => index.serialize(bytes),
        }
    }
}

impl Serialize for Attribute {
    fn serialize(&self, bytes: &mut Vec<u8>) -> Result<(), Error> {
        self.name_index.serialize(bytes)?;
        let mut buf = Vec::new();
        self.info.serialize(&mut buf)?;
        (buf.len() as u32).serialize(bytes)?;
        bytes.write_all(&buf[..])
    }
}

impl Serialize for JavaClass {
    fn serialize(&self, bytes: &mut Vec<u8>) -> Result<(), Error> {
        self.magic_bytes.serialize(bytes)?;
        self.minor_version.serialize(bytes)?;
        self.major_version.serialize(bytes)?;
        self.constant_pool.serialize(bytes)?;
        self.access_flags.serialize(bytes)?;
        self.this_class.serialize(bytes)?;
        self.super_class.unwrap_or(CPIndex(0)).serialize(bytes)?;
        self.interfaces.serialize(bytes)?;
        self.fields.serialize(bytes)?;
        self.methods.serialize(bytes)?;
        self.attributes.serialize(bytes)
    }
}
