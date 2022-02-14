#![allow(dead_code)]
use bitflags::bitflags;
use clap::{Parser, Subcommand};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{Cursor, Error, ErrorKind, BufReader},
    ops::Deref,
    path::{Path, PathBuf},
};

mod deserialization;
mod serialization;
mod display;

use deserialization::Deserialize;
use serialization::Serialize;

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, serde::Deserialize, serde::Serialize)]
struct CPIndex(u16);

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
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

#[derive(Debug, serde::Deserialize, serde::Serialize)]
enum ConstantPoolEntry {
    Class {
        name_index: CPIndex,
    },
    FieldRef {
        class_index: CPIndex,
        name_and_type_index: CPIndex,
    },
    MethodRef {
        class_index: CPIndex,
        name_and_type_index: CPIndex,
    },
    InterfaceMethodRef {
        class_index: CPIndex,
        name_and_type_index: CPIndex,
    },
    String {
        string_index: CPIndex,
    },
    Integer(i32),
    Float(f32),
    Long(i64),
    Double(f64),
    NameAndType {
        name_index: CPIndex,
        descriptor_index: CPIndex,
    },
    Utf8(String),
    MethodHandle {
        reference_kind: ReferenceKind,
        reference_index: CPIndex,
    },
    MethodType {
        descriptor_index: CPIndex,
    },
    InvokeDynamic {
        bootstrap_method_attr_index: u16,
        name_and_type_index: CPIndex,
    },
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct ConstantPool {
    // HashMap and not Vec, because ConstantPoolEntry's indices begin at 1, and some indices are
    // invalid (i.e with Double and Long constants).
    inner: HashMap<CPIndex, ConstantPoolEntry>,
}

bitflags! {
    #[derive(serde::Deserialize, serde::Serialize)]
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

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct Field {
    access_flags: AccessFlags,
    name_index: CPIndex,
    descriptor_index: CPIndex,
    attributes: Vec<Attribute>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct Method {
    access_flags: AccessFlags,
    name_index: CPIndex,
    descriptor_index: CPIndex,
    attributes: Vec<Attribute>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct ExceptionTableEntry {
    start: u16,
    end: u16,
    handler: u16,
    catch_type: CPIndex,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct CodeByte(u8);

#[derive(Debug, serde::Deserialize, serde::Serialize)]
enum AttributeInfo {
    Any(Vec<u8>),
    ConstantValue {
        index: CPIndex,
    },
    Code {
        max_stack: u16,
        max_locals: u16,
        code: Vec<CodeByte>,
        exception_table: Vec<ExceptionTableEntry>,
        // nested attributes yay
        attributes: Vec<Attribute>,
    },
    Exceptions {
        exception_index_table: Vec<CPIndex>,
    },
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct Attribute {
    name_index: CPIndex,
    info: AttributeInfo,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
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

impl CPIndex {
    fn none() -> Self {
        Self(0)
    }
}

impl TryFrom<u16> for CPIndex {
    type Error = ();
    fn try_from(v: u16) -> Result<Self, Self::Error> {
        match v {
            0 => Err(()),
            _ => Ok(Self(v)),
        }
    }
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
            _ => Err(()),
        }
    }
}

impl Into<u8> for ReferenceKind {
    fn into(self) -> u8 {
        self as u8
    }
}

impl<'a> ConstantPoolEntry {
    // returns the 'size' of this entry, because some java is weird
    fn size(&self) -> u16 {
        match self {
            ConstantPoolEntry::Long(_) | ConstantPoolEntry::Double(_) => 2u16,
            _ => 1u16,
        }
    }
}

impl Deref for ConstantPool {
    type Target = HashMap<CPIndex, ConstantPoolEntry>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl ConstantPool {
    fn size(&self) -> u16 {
        self.iter().map(|(_, v)| v.size()).fold(1u16, |a, v| a + v)
    }
}

impl std::ops::Index<CPIndex> for ConstantPool {
    type Output = ConstantPoolEntry;

    fn index(&self, index: CPIndex) -> &Self::Output {
        self.inner.get(&index).unwrap()
    }
}

impl<'a> Attribute {
    fn resolve(&mut self, cp: &ConstantPool) -> Result<(), Error> {
        if let AttributeInfo::Any(ref a) = self.info {
            let _size = a.len();
            let mut bytes = Cursor::new(a.clone());
            let bytes = &mut bytes;

            if let ConstantPoolEntry::Utf8(ref name) = cp[self.name_index] {
                let info = match name.as_str() {
                    "ConstantValue" => Ok(AttributeInfo::ConstantValue {
                        index: CPIndex::deserialize(bytes)?,
                    }),
                    "Code" => {
                        let max_stack = u16::deserialize(bytes)?;
                        let max_locals = u16::deserialize(bytes)?;

                        let code_length = u32::deserialize(bytes)?;
                        let mut code = Vec::with_capacity(code_length as usize);
                        for _ in 0..code_length {
                            code.push(CodeByte::deserialize(bytes)?);
                        }

                        let exception_table = Vec::<ExceptionTableEntry>::deserialize(bytes)?;
                        let mut attributes = Vec::<Attribute>::deserialize(bytes)?;

                        for a in attributes.iter_mut() {
                            let _ = a.resolve(cp);
                        }

                        Ok(AttributeInfo::Code {
                            max_stack,
                            max_locals,
                            code,
                            exception_table,
                            attributes,
                        })
                    }
                    "Exceptions" => Ok(AttributeInfo::Exceptions {
                        exception_index_table: Vec::<CPIndex>::deserialize(bytes)?,
                    }),
                    _ => Err(Error::new(ErrorKind::Other, "unkown attribute")),
                };
                let info = info?;

                self.info = info;
                Ok(())
            } else {
                Err(Error::new(
                    ErrorKind::Other,
                    "Error when trying to access Attribute name.",
                ))
            }
        } else {
            // already resolved
            Ok(())
        }
    }
}

impl JavaClass {
    fn from_file<P: AsRef<Path>>(file: P) -> Result<Self, Error> {
        let bytes = fs::read(file)?;
        let mut cursor = Cursor::new(bytes);
        Ok(JavaClass::deserialize(&mut cursor)?)
    }
    fn to_file<P: AsRef<Path>>(&self, file: P) -> Result<(), Error> {
        let mut buf = Vec::new();
        self.serialize(&mut buf)?;
        fs::write(file, buf)
    }
    fn print(&self) {
        println!("JavaClass {{");
        println!("--magic_bytes: {:08X}", self.magic_bytes);
        println!("--version: {}.{}", self.major_version, self.minor_version);
        println!("");
        println!("--ConstantPool:");
        let mut entries = self
            .constant_pool
            .iter()
            .collect::<Vec<(&CPIndex, &ConstantPoolEntry)>>();
        entries.sort_by(|(a, _), (b, _)| a.cmp(&b));
        for (k, v) in entries {
            println!("      {}: {}", k, v.display(&self.constant_pool));
        }
        println!("");
        println!("--This Class:");
        println!("    access_flags: {:?}", self.access_flags);
        println!(
            "    this_class: {}",
            self.this_class.display(&self.constant_pool)
        );
        println!(
            "    super_class: {}",
            self.super_class
                .unwrap_or(CPIndex(0))
                .display(&self.constant_pool)
        );
        println!("");
        println!("--Interfaces:");
        for i in self.interfaces.iter() {
            println!("    {:?}", i);
        }
        println!("");
        println!("--Fields:");
        for i in self.fields.iter() {
            println!(
                "    {}: {:?} ({})",
                i.name_index.display(&self.constant_pool),
                i.access_flags,
                i.descriptor_index.display(&self.constant_pool)
            );
            for j in i.attributes.iter() {
                println!("      {}", j.display(&self.constant_pool));
            }
        }
        println!("");
        println!("--Methods:");
        for i in self.methods.iter() {
            println!(
                "    {}: {:?} ({})",
                i.name_index.display(&self.constant_pool),
                i.access_flags,
                i.descriptor_index.display(&self.constant_pool)
            );
            for j in i.attributes.iter() {
                println!("      {}", j.display(&self.constant_pool));
            }
        }
        println!("");
        println!("--Attributes:");
        for i in self.attributes.iter() {
            println!("  {}", i.display(&self.constant_pool));
        }
        println!("}}");
    }
}

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    /// convert class file to json
    Json {
        /// path to the class
        #[clap(parse(from_os_str))]
        class: PathBuf,
        /// path to the json
        #[clap(parse(from_os_str))]
        json: PathBuf
    },
    /// convert json file to class
    Class {
        /// path to the json
        #[clap(parse(from_os_str))]
        json: PathBuf,
        /// path to the class
        #[clap(parse(from_os_str))]
        class: PathBuf
    }
}

fn main() {
    let args = Args::parse();

    match args.command {
        Command::Json { class, json } => {
            let cls = JavaClass::from_file(class).unwrap();
            fs::write(json, serde_json::to_string_pretty(&cls).unwrap()).unwrap();
        }
        Command::Class { json, class } => {
            let file = File::open(json).unwrap();
            let reader = BufReader::new(file);
            let cls: JavaClass = serde_json::from_reader(reader).unwrap();
            cls.to_file(class).unwrap();
        }
    }
}
