//! 读取与生成基岩版NBT结构
//!
//! # Example
//!
//! 读取NBT并打印
//!
//! ```
//! use crate::*;
//! fn read_example() -> IResult<()> {
//!     println!(
//!         "{}",
//!         NBT::from_path(Path::new("res/gold_farm.mcstructure"))?
//!     );
//!     Ok(())
//! }
//! ```
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use ritelinked::linked_hash_map::LinkedHashMap as Map;
use std::{
    fmt::{Debug, Display},
    io::{Cursor, Read, Seek, Write},
};
use thiserror::Error;
pub type Cur<'a> = Cursor<&'a mut Vec<u8>>;
pub type IResult<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO错误: {0}")]
    IO(#[from] std::io::Error),
    #[error("Utf8错误: {0}")]
    Utf8(#[source] std::string::FromUtf8Error, u64),
    #[error("没有根Compound标签, 错误的标签: {0}")]
    Root(u8),
    #[error("无效的类型ID: {0}")]
    InvalidTypeId(u8),
    #[error("List标签中的类型不唯一")]
    HeterogeneousList,
    #[error("List标签中的类型不唯一")]
    FmtError(#[source] std::fmt::Error),
    #[error("{0}")]
    Unknown(String),
}
///表示一个NBT结构及名称
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde_rs", derive(serde::Serialize, serde::Deserialize))]
pub struct NBT {
    pub name: String,
    pub data: Value,
}

impl NBT {
    ///从字节流中读取数据返回[`NBT`]
    pub fn new(bytes: &mut Vec<u8>) -> IResult<NBT> {
        let mut c = Cursor::new(bytes);
        Ok(NBT::read(&mut c)?)
    }

    pub fn named(name: &str) -> IResult<NBT> {
        Ok(NBT {
            name: name.to_owned(),
            data: Value::Compound(Map::new()),
        })
    }

    pub fn from_reader<R: Read>(r: &mut R) -> IResult<NBT> {
        let mut buf: Vec<u8> = Vec::new();
        r.read_to_end(&mut buf)?;
        let mut c = Cursor::new(&mut buf);
        Ok(NBT::read(&mut c)?)
    }

    ///向字节流中写入NBT数据
    pub fn write<W: Write>(&self, vec: &mut W, bedrock_header: bool) -> IResult<()> {
        let mut buf = Vec::<u8>::new();
        buf.write_u8(0x0a)?;
        write_string(&mut buf, &self.name)?;

        self.data.write(&mut buf)?;

        if bedrock_header {
            vec.write_i32::<LE>(0x08)?;
            vec.write_u32::<LE>(buf.len() as u32)?;
        }

        vec.write_all(buf.as_slice())?;
        Ok(())
    }

    pub fn header<R: Read>(r: &mut R) -> IResult<Option<[u8; 8]>> {
        let mut header = [0u8; 8];
        let g = match r.read_exact(&mut header) {
            Ok(_) => Some(header),
            Err(_) => None,
        };
        return Ok(g);
    }

    #[inline]
    fn read(c: &mut Cur) -> IResult<NBT> {
        if c.read_i32::<LE>()? == 0x08 {
            c.seek(std::io::SeekFrom::Start(8))?;
        } else {
            c.seek(std::io::SeekFrom::Start(0))?;
        }
        let (tag, name) = read_next_header(c)?;

        if tag != 0x0a {
            return Err(Error::Root(tag));
        }

        Ok(NBT {
            name,
            data: Value::read(tag, c)?,
        })
    }
}

///NBT标签的枚举
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde_rs", derive(serde::Serialize, serde::Deserialize))]
pub enum Value {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(Vec<i8>),
    String(String),
    List(Vec<Value>),
    Compound(Map<String, Value>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.print(f, 0)
    }
}

impl Display for NBT {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut vec = Vec::new();
        write!(f, "Name: \"{}\"\n", self.name)?;
        write!(f, "Header: ")?;

        if let Err(_) = self.write(&mut vec, true) {
            panic!()
        }

        match NBT::header(&mut vec.as_slice()) {
            Ok(h) => match h {
                Some(b) => {
                    write!(f, "[")?;

                    for (index, item) in b.iter().enumerate() {
                        if index > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{:#04X}", item)?;
                    }

                    write!(f, "]")
                }
                None => write!(f, "None"),
            },
            Err(e) => {
                panic!("{e}");
            }
        }?;
        write!(f, "\n")?;
        self.data.print(f, 0)
    }
}

impl Value {
    pub fn tag(&self) -> u8 {
        match *self {
            Value::Byte(_) => 0x01,
            Value::Short(_) => 0x02,
            Value::Int(_) => 0x03,
            Value::Long(_) => 0x04,
            Value::Float(_) => 0x05,
            Value::Double(_) => 0x06,
            Value::ByteArray(_) => 0x07,
            Value::String(_) => 0x08,
            Value::List(_) => 0x09,
            Value::Compound(_) => 0x0a,
            Value::IntArray(_) => 0x0b,
            Value::LongArray(_) => 0x0c,
        }
    }

    pub fn read(tag: u8, c: &mut Cur) -> IResult<Value> {
        match tag {
            0x01 => Ok(Value::Byte(c.read_i8()?)),
            0x02 => Ok(Value::Short(c.read_i16::<LE>()?)),
            0x03 => Ok(Value::Int(c.read_i32::<LE>()?)),
            0x04 => Ok(Value::Long(c.read_i64::<LE>()?)),
            0x05 => Ok(Value::Float(c.read_f32::<LE>()?)),
            0x06 => Ok(Value::Double(c.read_f64::<LE>()?)),
            0x07 => {
                let len = c.read_i32::<LE>()? as usize;
                let mut buf = Vec::with_capacity(len);
                for _ in 0..len {
                    buf.push(c.read_i8()?);
                }
                Ok(Value::ByteArray(buf))
            }
            0x08 => Ok(Value::String(read_string(c)?)),
            0x09 => {
                let id = c.read_u8()?;
                let len = c.read_i32::<LE>()? as usize;
                let mut buf = Vec::with_capacity(len);
                for _ in 0..len {
                    buf.push(Value::read(id, c)?);
                }
                Ok(Value::List(buf))
            }
            0x0a => {
                let mut buf = Map::new();
                loop {
                    let (id, name) = read_next_header(c)?;
                    if id == 0x00 {
                        break;
                    }
                    let tag = Value::read(id, c)?;
                    buf.insert(name, tag);
                }
                Ok(Value::Compound(buf))
            }
            0x0b => {
                let len = c.read_i32::<LE>()? as usize;
                let mut buf = Vec::with_capacity(len);
                for _ in 0..len {
                    buf.push(c.read_i32::<LE>()?);
                }
                Ok(Value::IntArray(buf))
            }
            0x0c => {
                let len = c.read_i32::<LE>()? as usize;
                let mut buf = Vec::with_capacity(len);
                for _ in 0..len {
                    buf.push(c.read_i64::<LE>()?);
                }
                Ok(Value::LongArray(buf))
            }
            e => Err(Error::InvalidTypeId(e)),
        }
    }

    pub fn write(&self, c: &mut Vec<u8>) -> IResult<()> {
        match *self {
            Value::Byte(v) => c.write_i8(v)?,
            Value::Short(v) => c.write_i16::<LE>(v)?,
            Value::Int(v) => c.write_i32::<LE>(v)?,
            Value::Long(v) => c.write_i64::<LE>(v)?,
            Value::Float(v) => c.write_f32::<LE>(v)?,
            Value::Double(v) => c.write_f64::<LE>(v)?,
            Value::ByteArray(ref v) => {
                c.write_i32::<LE>(v.len() as i32)?;
                for &v in v {
                    c.write_i8(v)?;
                }
            }
            Value::String(ref v) => write_string(c, v)?,
            Value::List(ref v) => {
                if v.is_empty() {
                    c.write_u8(0)?;
                    c.write_i32::<LE>(0)?;
                } else {
                    let first_id = v[0].tag();
                    c.write_u8(first_id)?;
                    c.write_i32::<LE>(v.len() as i32)?;
                    for nbt in v {
                        if nbt.tag() != first_id {
                            return Err(Error::HeterogeneousList);
                        }
                        nbt.write(c)?;
                    }
                }
            }
            Value::Compound(ref v) => {
                for (name, nbt) in v {
                    c.write_u8(nbt.tag())?;
                    write_string(c, name)?;
                    nbt.write(c)?;
                }
                c.write_u8(0)?;
            }
            Value::IntArray(ref v) => {
                c.write_i32::<LE>(v.len() as i32)?;
                for &v in v {
                    c.write_i32::<LE>(v)?;
                }
            }
            Value::LongArray(ref v) => {
                c.write_i32::<LE>(v.len() as i32)?;
                for &v in v {
                    c.write_i64::<LE>(v)?;
                }
            }
        }
        Ok(())
    }

    pub fn tag_name(&self) -> &str {
        match *self {
            Value::Byte(_) => "TAG_Byte",
            Value::Short(_) => "TAG_Short",
            Value::Int(_) => "TAG_Int",
            Value::Long(_) => "TAG_Long",
            Value::Float(_) => "TAG_Float",
            Value::Double(_) => "TAG_Double",
            Value::ByteArray(_) => "TAG_ByteArray",
            Value::String(_) => "TAG_String",
            Value::List(_) => "TAG_List",
            Value::Compound(_) => "TAG_Compound",
            Value::IntArray(_) => "TAG_IntArray",
            Value::LongArray(_) => "TAG_LongArray",
        }
    }

    pub fn print(&self, f: &mut std::fmt::Formatter, offset: usize) -> std::fmt::Result {
        match *self {
            Value::Byte(v) => write!(f, "{}", v),
            Value::Short(v) => write!(f, "{}", v),
            Value::Int(v) => write!(f, "{}", v),
            Value::Long(v) => write!(f, "{}", v),
            Value::Float(v) => write!(f, "{}", v),
            Value::Double(v) => write!(f, "{}", v),
            Value::ByteArray(ref v) => write!(f, "{:?}", v),
            Value::String(ref v) => write!(f, "{}", v),
            Value::IntArray(ref v) => write!(f, "{:?}", v),
            Value::LongArray(ref v) => write!(f, "{:?}", v),
            Value::List(ref v) => {
                if v.is_empty() {
                    write!(f, "zero entries")
                } else {
                    write!(
                        f,
                        "{} entries of type {}\n{:>width$}\n",
                        v.len(),
                        v[0].tag_name(),
                        "{",
                        width = offset + 1
                    )?;
                    for tag in v {
                        let new_offset = offset + 2;
                        write!(
                            f,
                            "{:>width$}(None): ",
                            tag.tag_name(),
                            width = new_offset + tag.tag_name().len()
                        )?;
                        tag.print(f, new_offset)?;
                        writeln!(f)?;
                    }
                    write!(f, "{:>width$}", "}", width = offset + 1)
                }
            }
            Value::Compound(ref v) => {
                write!(
                    f,
                    "{} entry(ies)\n{:>width$}\n",
                    v.len(),
                    "{",
                    width = offset + 1
                )?;
                for (name, tag) in v {
                    let new_offset = offset + 2;
                    write!(
                        f,
                        "{:>width$}({}): ",
                        tag.tag_name(),
                        name,
                        width = new_offset + tag.tag_name().len()
                    )?;
                    tag.print(f, new_offset)?;
                    writeln!(f)?;
                }
                write!(f, "{:>width$}", "}", width = offset + 1)
            }
        }
    }
}

fn read_next_header(c: &mut Cur) -> IResult<(u8, String)> {
    let tag = c.read_u8()?;

    return if tag == 0x00 {
        Ok((0x00, "".to_string()))
    } else {
        Ok((tag, read_string(c)?))
    };
}

#[inline]
fn read_string(c: &mut Cur) -> IResult<String> {
    let len = c.read_u16::<LE>()?;

    if len == 0 {
        return Ok("".into());
    }

    let mut buf = vec![0; len.into()];

    c.read_exact(buf.as_mut_slice())?;

    let string = match String::from_utf8(buf) {
        Err(e) => return Err(Error::Utf8(e, c.position())),
        Ok(s) => s,
    };
    Ok(string)
}

#[inline]
fn write_string(c: &mut Vec<u8>, s: &str) -> IResult<()> {
    let b = s.as_bytes();
    c.write_u16::<LE>(s.len() as u16)?;
    c.write_all(b)?;
    Ok(())
}

//#[cfg(not(feature = "wasm"))]
mod tests;

/// # Wasm功能
///
/// ## 示例 (javascript)
///
/// ``` javascript
/// let nbt = NBT.from(
///   new Uint8Array([8, 0, 0, 0, 36, 0, 0, 0, 10, 4, 0, 84, 101, 115, 116, 1, 3, 0, 107, 101, 121, 8, 11, 2, 0, 106, 115, 3, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 0])
/// );
///
/// nbt.set('key');
/// nbt.set('js');
///
/// nbt.set('obj/obj2/String', {
///     String: 'string from js'
/// });
///
/// nbt.set('obj/obj2/String', {
///     String: 'replace'
/// });
///
/// nbt.set('obj/obj2/Byte', {
///     Byte: 8
/// });
///
/// nbt.set('obj/LongArray', {
///     LongArray: [8, -9, 0, 1816]
/// });
///
/// console.log(nbt.value.data.Compound);
/// console.log(nbt.bytes(true));
/// ```
#[cfg(feature = "wasm")]
pub mod wasm;
