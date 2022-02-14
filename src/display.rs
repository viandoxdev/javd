use std::fmt::Display;

use crate::{CPIndex, ConstantPool, ReferenceKind, ConstantPoolEntry, Attribute, AttributeInfo};

pub struct DisplayCP<'a>(CPIndex, &'a ConstantPool);
pub struct DisplayConstantPoolEntry<'a>(&'a ConstantPoolEntry, &'a ConstantPool);
pub struct DisplayAttribute<'a>(&'a Attribute, &'a ConstantPool);

impl<'a> Display for DisplayCP<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.1.get(&self.0) {
            Some(v) => write!(f, "{}", v.display(self.1))?,
            None => write!(f, "(NONE)")?,
        };
        //write!(f, "@{}", self.0)?;
        Ok(())
    }
}

impl<'a> Display for DisplayConstantPoolEntry<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            ConstantPoolEntry::Integer(i) => write!(f, "(int {})", i),
            ConstantPoolEntry::Long(l) => write!(f, "(long {})", l),
            ConstantPoolEntry::Utf8(s) => write!(f, "'{}'", s),
            ConstantPoolEntry::Float(d) => write!(f, "(float {})", d),
            ConstantPoolEntry::Double(d) => write!(f, "(double {})", d),
            ConstantPoolEntry::Class { name_index } => {
                write!(f, "(class {})", name_index.display(self.1))
            }
            ConstantPoolEntry::String { string_index } => {
                write!(f, "(string {})", string_index.display(self.1))
            }
            ConstantPoolEntry::FieldRef {
                class_index,
                name_and_type_index,
            } => write!(
                f,
                "(fieldref {} {})",
                class_index.display(self.1),
                name_and_type_index.display(self.1)
            ),
            ConstantPoolEntry::MethodRef {
                class_index,
                name_and_type_index,
            } => write!(
                f,
                "(methodref {} {})",
                class_index.display(self.1),
                name_and_type_index.display(self.1)
            ),
            ConstantPoolEntry::InterfaceMethodRef {
                class_index,
                name_and_type_index,
            } => write!(
                f,
                "(interfacemethodref {} {})",
                class_index.display(self.1),
                name_and_type_index.display(self.1)
            ),
            ConstantPoolEntry::MethodType { descriptor_index } => {
                write!(f, "(methodtype {})", descriptor_index.display(self.1))
            }
            ConstantPoolEntry::NameAndType {
                name_index,
                descriptor_index,
            } => write!(
                f,
                "(name {} {})",
                name_index.display(self.1),
                descriptor_index.display(self.1)
            ),
            ConstantPoolEntry::MethodHandle {
                reference_kind,
                reference_index,
            } => write!(
                f,
                "(kind {} {})",
                reference_kind,
                reference_index.display(self.1)
            ),
            ConstantPoolEntry::InvokeDynamic {
                bootstrap_method_attr_index,
                name_and_type_index,
            } => write!(
                f,
                "(invokedyn attr {} {})",
                bootstrap_method_attr_index,
                name_and_type_index.display(self.1)
            ),
        }
    }
}

impl<'a> Display for DisplayAttribute<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {}",
            self.0.name_index.display(&self.1),
            self.0.info
        )
    }
}

impl Display for CPIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04}", self.0)
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

impl Display for AttributeInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<'a> CPIndex {
    pub fn display(&self, cp: &'a ConstantPool) -> DisplayCP<'a> {
        DisplayCP(*self, cp)
    }
}

impl<'a> ConstantPoolEntry {
    pub fn display(&'a self, cp: &'a ConstantPool) -> DisplayConstantPoolEntry<'a> {
        DisplayConstantPoolEntry(self, cp)
    }
}

impl<'a> Attribute {
    pub fn display(&'a self, cp: &'a ConstantPool) -> DisplayAttribute<'a> {
        DisplayAttribute(self, cp)
    }
}
