#![allow(dead_code)]
use std::{fs, io::{Cursor, Error, ErrorKind}, collections::HashMap, ops::Deref, fmt::Display};
use bitflags::bitflags;
use read::read_u8;

mod read;

trait Deserialize {
    fn deserialize(bytes: &mut Cursor<Vec<u8>>) -> Result<Self, Error> where Self: Sized;
}

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
struct CPIndex(u16);

impl Display for CPIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04}", self.0)
    }
}

impl TryFrom<u16> for CPIndex {
    type Error = ();
    fn try_from(v: u16) -> Result<Self, Self::Error> {
        match v {
            0 => Err(()),
            _ => Ok(Self(v))
        }
    }
}

impl Deserialize for CPIndex {
    fn deserialize(bytes: &mut Cursor<Vec<u8>>) -> Result<Self, Error> {
        read::read_u16(bytes)?.try_into()
            .map_err(|_| Error::new(ErrorKind::Other, "Error when trying to convert u16 to CPIndex (value is 0)."))
    }
}

#[derive(Debug)]
enum ReferenceKind {
    GetField = 1,
    GetStatic = 2,
    PutField = 3,
    PutStatic = 4,
    InvokeVirtual = 5,
    InvokeStatic = 6,
    InvokeSpecial = 7,
    NewInvokeSpecial = 8,
    InvokeInterface = 9,
}

impl TryFrom<u8> for ReferenceKind {
    type Error = ();
    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            1 => Ok(ReferenceKind::GetField),
            2 => Ok(ReferenceKind::GetStatic),
            3 => Ok(ReferenceKind::PutField),
            4 => Ok(ReferenceKind::PutStatic),
            5 => Ok(ReferenceKind::InvokeVirtual),
            6 => Ok(ReferenceKind::InvokeStatic),
            7 => Ok(ReferenceKind::InvokeSpecial),
            8 => Ok(ReferenceKind::NewInvokeSpecial),
            9 => Ok(ReferenceKind::InvokeInterface),
            _ => Err(())
        }
    }
}

impl Display for ReferenceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReferenceKind::GetField => write!(f, "GetField"),
            ReferenceKind::GetStatic => write!(f, "GetStatic"),
            ReferenceKind::PutField => write!(f, "PutField"),
            ReferenceKind::PutStatic => write!(f, "PutStatic"),
            ReferenceKind::InvokeVirtual => write!(f, "InvokeVirtual"),
            ReferenceKind::InvokeStatic => write!(f, "InvokeStatic"),
            ReferenceKind::InvokeSpecial => write!(f, "InvokeSpecial"),
            ReferenceKind::NewInvokeSpecial => write!(f, "NewInvokeSpecial"),
            ReferenceKind::InvokeInterface => write!(f, "InvokeInterface"),
        }
    }
}

impl Into<u8> for ReferenceKind {
    fn into(self) -> u8 {
        self as u8
    }
}

impl Deserialize for ReferenceKind {
    fn deserialize(bytes: &mut Cursor<Vec<u8>>) -> Result<Self, Error> {
        read::read_u8(bytes)?.try_into()
            .map_err(|_| Error::new(ErrorKind::Other, "Error when trying to convert u8 to ReferenceKind"))
    }
}

#[derive(Debug)]
enum ConstantPoolEntry {
    Class {
        name_index: CPIndex
    },
    FieldRef {
        class_index: CPIndex,
        name_and_type_index: CPIndex
    },
    MethodRef {
        class_index: CPIndex,
        name_and_type_index: CPIndex
    },
    InterfaceMethodRef {
        class_index: CPIndex,
        name_and_type_index: CPIndex
    },
    String {
        string_index: CPIndex
    },
    Integer(i32),
    Float(f32),
    Long(i64),
    Double(f64),
    NameAndType {
        name_index: CPIndex,
        descriptor_index: CPIndex
    },
    Utf8(String),
    MethodHandle {
        reference_kind: ReferenceKind,
        reference_index: CPIndex
    },
    MethodType {
        descriptor_index: CPIndex
    },
    InvokeDynamic {
        bootstrap_method_attr_index: u16,
        name_and_type_index: CPIndex
    },
}

impl ConstantPoolEntry {
    // returns the 'size' of this entry, because some java is weird 
    fn size(&self) -> u16 {
        match self {
            ConstantPoolEntry::Long(_) | ConstantPoolEntry::Double(_) => 2u16,
            _ => 1u16
        }
    }
}

impl Deserialize for ConstantPoolEntry {
    fn deserialize(bytes: &mut Cursor<Vec<u8>>) -> Result<Self, Error> {
        let tag = read_u8(bytes)?;
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
                name_and_type_index: CPIndex::deserialize(bytes)?
            }),
            08 => Ok(ConstantPoolEntry::String {
                string_index: CPIndex::deserialize(bytes)?
            }),
            03 => Ok(ConstantPoolEntry::Integer(read::read_i32(bytes)?)),
            04 => Ok(ConstantPoolEntry::Float(read::read_f32(bytes)?)),
            05 => Ok(ConstantPoolEntry::Long(read::read_i64(bytes)?)),
            06 => Ok(ConstantPoolEntry::Double(read::read_f64(bytes)?)),
            12 => Ok(ConstantPoolEntry::NameAndType {
                name_index: CPIndex::deserialize(bytes)?,
                descriptor_index: CPIndex::deserialize(bytes)?
            }),
            01 => {
                let length = read::read_u16(bytes)?;
                let buf = read::read_bytes(bytes, length as usize)?;
                Ok(ConstantPoolEntry::Utf8(String::from_utf8_lossy(&buf).into()))
            }
            15 => Ok(ConstantPoolEntry::MethodHandle {
                reference_kind: ReferenceKind::deserialize(bytes)?,
                reference_index: CPIndex::deserialize(bytes)?
            }),
            16 => Ok(ConstantPoolEntry::MethodType {
                descriptor_index: CPIndex::deserialize(bytes)?
            }),
            18 => Ok(ConstantPoolEntry::InvokeDynamic {
                bootstrap_method_attr_index: read::read_u16(bytes)?,
                name_and_type_index: CPIndex::deserialize(bytes)?
            }),
            _ => Err(Error::new(ErrorKind::Other, "Unkown tag on ConstantPoolEntry"))
        }
    }
}

struct DisplayConstantPoolEntry<'a>(&'a ConstantPoolEntry,&'a ConstantPool);

impl<'a> Display for DisplayConstantPoolEntry<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            ConstantPoolEntry::Integer(i)
                => write!(f, "(int {})", i),
            ConstantPoolEntry::Long(l)
                => write!(f, "(long {})", l),
            ConstantPoolEntry::Utf8(s)
                => write!(f, "'{}'", s),
            ConstantPoolEntry::Float(d)
                => write!(f, "(float {})", d),
            ConstantPoolEntry::Double(d)
                => write!(f, "(double {})", d),
            ConstantPoolEntry::Class { name_index }
                => write!(f, "(class {})", DisplayCP(*name_index, self.1)),
            ConstantPoolEntry::String { string_index }
                => write!(f, "(string {})", DisplayCP(*string_index, self.1)),
            ConstantPoolEntry::FieldRef { class_index, name_and_type_index }
                => write!(f, "(fieldref {} {})", DisplayCP(*class_index, self.1), DisplayCP(*name_and_type_index, self.1)),
            ConstantPoolEntry::MethodRef { class_index, name_and_type_index }
                => write!(f, "(methodref {} {})", DisplayCP(*class_index, self.1), DisplayCP(*name_and_type_index, self.1)),
            ConstantPoolEntry::InterfaceMethodRef { class_index, name_and_type_index }
                => write!(f, "(interfacemethodref {} {})", DisplayCP(*class_index, self.1), DisplayCP(*name_and_type_index, self.1)),
            ConstantPoolEntry::MethodType { descriptor_index }
                => write!(f, "(methodtype {})", DisplayCP(*descriptor_index, self.1)),
            ConstantPoolEntry::NameAndType { name_index, descriptor_index }
                => write!(f, "(name {} {})", DisplayCP(*name_index, self.1), DisplayCP(*descriptor_index, self.1)),
            ConstantPoolEntry::MethodHandle { reference_kind, reference_index }
                => write!(f, "(kind {} {})", reference_kind, DisplayCP(*reference_index, self.1)),
            ConstantPoolEntry::InvokeDynamic { bootstrap_method_attr_index, name_and_type_index }
                => write!(f, "(invokedyn attr {} {})", bootstrap_method_attr_index, DisplayCP(*name_and_type_index, self.1)),
        }
    }
}

#[derive(Debug)]
struct ConstantPool {
    // HashMap and not Vec, because ConstantPoolEntry's indices begin at 1, and some indices are
    // invalid (i.e with Double and Long constants).
    inner: HashMap<CPIndex, ConstantPoolEntry>
}

impl Deref for ConstantPool {
    type Target = HashMap<CPIndex, ConstantPoolEntry>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Deserialize for ConstantPool {
    fn deserialize(bytes: &mut Cursor<Vec<u8>>) -> Result<ConstantPool, Error> {
        let count = read::read_u16(bytes)?;
        let mut index = 1u16; // indices starts at 1
        let mut map = HashMap::new();
        
        while index < count {
            let entry = ConstantPoolEntry::deserialize(bytes)?;
            let size = entry.size();

            map.insert(index.try_into().unwrap(), entry);
            index += size;
        }
        
        Ok(Self {
            inner: map
        })
    }
}

impl std::ops::Index<CPIndex> for ConstantPool {
    type Output = ConstantPoolEntry;

    fn index(&self, index: CPIndex) -> &Self::Output {
        self.inner.get(&index).unwrap()
    }
}

bitflags! {
    struct AccessFlags: u16 {
        const PUBLIC       = 0x0001; // ---- ---- ---- ---1
        const PRIVATE      = 0x0002; // ---- ---- ---- --1-
        const PROTECTED    = 0x0004; // ---- ---- ---- -1--
        const STATIC       = 0x0008; // ---- ---- ---- 1---
        const FINAL        = 0x0010; // ---- ---- ---1 ----
        const SUPER        = 0x0020; // ---- ---- --1- ----
        const SYNCHRONIZED = 0x0020;
        const VOLATILE     = 0x0040; // ---- ---- -1-- ----
        const BRIDGE       = 0x0040;
        const TRANSIENT    = 0x0080; // ---- ---- 1--- ----
        const VARARGS      = 0x0080;
        const NATIVE       = 0x0100; // ---- ---1 ---- ----
        const INTERFACE    = 0x0200; // ---- --1- ---- ----
        const ABSTRACT     = 0x0400; // ---- -1-- ---- ----
        const STRICT       = 0x0800; // ---- 1--- ---- ----
        const SYNTHETIC    = 0x1000; // ---1 ---- ---- ----
        const ANNOTATION   = 0x2000; // --1- ---- ---- ----
        const ENUM         = 0x4000; // -1-- ---- ---- ----
    }
}

impl Deserialize for AccessFlags {
    fn deserialize(bytes: &mut Cursor<Vec<u8>>) -> Result<Self, Error> {
        AccessFlags::from_bits(read::read_u16(bytes)?)
            .ok_or(Error::new(ErrorKind::Other, "Error when trying to convert u16 to AccessFlags"))
    }
}

impl<T> Deserialize for Vec<T> where T: Deserialize {
    fn deserialize(bytes: &mut Cursor<Vec<u8>>) -> Result<Self, Error> {
        let count = read::read_u16(bytes)? as usize;
        let mut res = Vec::with_capacity(count);

        for _ in 0..count {
            res.push(T::deserialize(bytes)?);
        }
        Ok(res)
    }
}

#[derive(Debug)]
struct Field {
    access_flags: AccessFlags,
    name_index: CPIndex,
    descriptor_index: CPIndex,
    attributes: Vec<Attribute>
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
            attributes
        })
    }
}

#[derive(Debug)]
struct Method {
    access_flags: AccessFlags,
    name_index: CPIndex,
    descriptor_index: CPIndex,
    attributes: Vec<Attribute>
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
            attributes
        })
    }
}

#[derive(Debug)]
enum AttributeInfo {
    Any(Vec<u8>)
}

#[derive(Debug)]
struct Attribute {
    name_index: CPIndex,
    info: AttributeInfo
}

impl Deserialize for AttributeInfo {
    fn deserialize(bytes: &mut Cursor<Vec<u8>>) -> Result<Self, Error> {
        let size = read::read_u32(bytes)?;
        let buf = read::read_bytes(bytes, size as usize)?;

        Ok(AttributeInfo::Any(buf))
    }
}

impl Deserialize for Attribute {
    fn deserialize(bytes: &mut Cursor<Vec<u8>>) -> Result<Self, Error> {
        let name_index = CPIndex::deserialize(bytes)?;
        let info = AttributeInfo::deserialize(bytes)?;

        Ok(Self {
            name_index, info
        })
    }
}

impl Display for Attribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "I'm an Attribute ! Yay")
    }
}

#[derive(Debug)]
struct JavaClass {
    magic_bytes: u32,
    minor_version: u16,
    major_version: u16,
    constant_pool: ConstantPool,
    access_flags: AccessFlags,
    this_class: CPIndex,
    super_class: Option<CPIndex>,
    interfaces: Vec<CPIndex>,
    fields: Vec<Field>,
    methods: Vec<Method>,
    attributes: Vec<Attribute>,
}

impl JavaClass {
    fn deserialize(bytes: Vec<u8>) -> Result<Self, Error> {
        let mut cursor = Cursor::new(bytes);

        let magic_bytes = read::read_u32(&mut cursor)?;
        let minor_version = read::read_u16(&mut cursor)?;
        let major_version = read::read_u16(&mut cursor)?;
        let constant_pool = ConstantPool::deserialize(&mut cursor)?;
        let access_flags = AccessFlags::deserialize(&mut cursor)?;
        let this_class = CPIndex::deserialize(&mut cursor)?;
        let super_class = CPIndex::deserialize(&mut cursor).ok(); // optional
        let interfaces = Vec::<CPIndex>::deserialize(&mut cursor)?;
        let fields = Vec::<Field>::deserialize(&mut cursor)?;
        let methods = Vec::<Method>::deserialize(&mut cursor)?;
        let attributes = Vec::<Attribute>::deserialize(&mut cursor)?;

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

struct DisplayCP<'a>(CPIndex, &'a ConstantPool);

impl<'a> Display for DisplayCP<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.1.get(&self.0) {
            Some(v) => write!(f, "{}", DisplayConstantPoolEntry(v, self.1))?,
            None => write!(f, "(NONE)")?
        };
        //write!(f, "@{}", self.0)
        Ok(())
    }
}

impl JavaClass {
    fn print(&self) {
        println!("JavaClass {{");
        println!("--magic_bytes: {:08X}", self.magic_bytes);
        println!("--version: {}.{}", self.major_version, self.minor_version);
        println!("");
        println!("--ConstantPool:");
        let mut entries = self.constant_pool.iter().collect::<Vec<(&CPIndex, &ConstantPoolEntry)>>();
        entries.sort_by(|(a, _), (b, _)| a.cmp(&b));
        for (k, v) in entries {
            println!("      {}: {}", k, DisplayConstantPoolEntry(v, &self.constant_pool));
        }
        println!("");
        println!("--This Class:");
        println!("    access_flags: {:?}", self.access_flags);
        println!("    this_class: {}", DisplayCP(self.this_class, &self.constant_pool));
        println!("    super_class: {}", DisplayCP(self.super_class.unwrap_or(CPIndex(0)), &self.constant_pool));
        println!("");
        println!("--Interfaces:");
        for i in self.interfaces.iter() {
            println!("    {:?}", i);
        }
        println!("");
        println!("--Fields:");
        for i in self.fields.iter() {
            println!("    {}: {:?} ({})", DisplayCP(i.name_index, &self.constant_pool), i.access_flags, DisplayCP(i.descriptor_index, &self.constant_pool));
            for j in i.attributes.iter() {
                println!("      {}", j);
            }
        }
        println!("");
        println!("--Methods:");
        for i in self.methods.iter() {
            println!("    {}: {:?} ({})", DisplayCP(i.name_index, &self.constant_pool), i.access_flags, DisplayCP(i.descriptor_index, &self.constant_pool));
            for j in i.attributes.iter() {
                println!("    {}", j);
            }
        }
        println!("");
        println!("--Attributes:");
        for i in self.attributes.iter() {
            println!("  {}", i);
        }
        println!("}}");
    }
}

fn main() {
    let bytes = fs::read("Main.class").unwrap();
    let class = JavaClass::deserialize(bytes).unwrap();
    class.print();
}
