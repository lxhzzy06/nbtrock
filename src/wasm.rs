use crate::{Error, IResult, Value};
use ritelinked::linked_hash_map::LinkedHashMap as Map;
use std::{fmt::Display, io::Cursor};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};
type WResult<T> = Result<T, WasmError>;

#[wasm_bindgen(start)]
fn init() {
    log("nbtrock: init");
    #[cfg(feature = "panic_hook")]
    {
        extern crate console_error_panic_hook;
        log("nbtrock: Set panic hook");
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    }
}

#[derive(Error, Debug)]
pub enum WasmError {
    #[error("序列化失败: {0}")]
    Serde(#[from] serde_wasm_bindgen::Error),
    #[error("无法将路径转换为utf8格式")]
    InvalidStr,
    #[error("无效的路径: {0}")]
    InvalidPath(String),
    #[error("{0}")]
    Error(#[source] Error),
}

#[wasm_bindgen(typescript_custom_section)]
const IByte: &'static str = r#"
    interface IByte {
       readonly Byte: number;
    }
"#;

#[wasm_bindgen(typescript_custom_section)]
const IShort: &'static str = r#"
    interface IShort {
       readonly Short: number;
    }
"#;

#[wasm_bindgen(typescript_custom_section)]
const IInt: &'static str = r#"
    interface IInt {
       readonly Int: number;
    }
"#;

#[wasm_bindgen(typescript_custom_section)]
const ILong: &'static str = r#"
    interface ILong {
       readonly Long: number;
    }
"#;

#[wasm_bindgen(typescript_custom_section)]
const IFloat: &'static str = r#"
    interface IFloat {
       readonly Float: number;
    }
"#;

#[wasm_bindgen(typescript_custom_section)]
const IDouble: &'static str = r#"
    interface IDouble {
       readonly Double: number;
    }
"#;

#[wasm_bindgen(typescript_custom_section)]
const IString: &'static str = r#"
    interface IString {
       readonly String: string;
    }
"#;

#[wasm_bindgen(typescript_custom_section)]
const IByteArray: &'static str = r#"
    interface IByteArray {
       readonly ByteArray: number[];
    }
"#;

#[wasm_bindgen(typescript_custom_section)]
const IIntArray: &'static str = r#"
    interface IIntArray {
       readonly IntArray: number[];
    }
"#;

#[wasm_bindgen(typescript_custom_section)]
const ILongArray: &'static str = r#"
    interface ILongArray {
       readonly LongArray: number[];
    }
"#;

#[wasm_bindgen(typescript_custom_section)]
const IList: &'static str = r#"
    interface IList {
       readonly List: IValue[];
    }
"#;

#[wasm_bindgen(typescript_custom_section)]
const ICompound: &'static str = r#"
    interface ICompound {
       readonly Compound: Map<string, IValue>;
    }
"#;

#[wasm_bindgen(typescript_custom_section)]
const IValue: &'static str = r#"
    type IValue = IByte | IShort | IInt | ILong | IFloat | IDouble | IString | IByteArray | IIntArray | ILongArray | IList | ICompound;
"#;

#[wasm_bindgen(typescript_custom_section)]
const IHeader: &'static str = r#"
    type IHeader = [number, number, number, number, number, number, number, number] | undefined;
"#;

#[wasm_bindgen(typescript_custom_section)]
const INBT: &'static str = r#"
    interface INBT {
       readonly name: string;
       readonly data: ICompound;
    }
"#;

#[wasm_bindgen]
extern "C" {
    /// TypeScript类型
    /// ```
    /// interface INBT {
    ///     readonly name: string;
    ///     readonly data: ICompound;
    ///  }
    /// ```
    #[wasm_bindgen(typescript_type = "INBT")]
    pub type INBT;
    ///TypeScript类型:
    /// ```
    /// type IValue = IByte | IShort | IInt | ILong | IFloat | IDouble | IString | IByteArray | IIntArray | ILongArray | IList | ICompound;
    /// ```
    #[wasm_bindgen(typescript_type = "IValue")]
    pub type IValue;
    ///TypeScript类型:
    /// ```
    /// type IHeader = [number, number, number, number, number, number, number, number] | undefined;
    /// ```
    #[wasm_bindgen(typescript_type = "IHeader")]
    pub type IHeader;

    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

///对于[`crate::NBT`]的包装，实现了一些方法用于与JavaScript通信
#[derive(Debug)]
#[wasm_bindgen]
pub struct NBT(#[wasm_bindgen(skip)] pub crate::NBT);

impl Into<JsValue> for Error {
    fn into(self) -> JsValue {
        Error::to_string(&self).into()
    }
}

impl Into<JsValue> for WasmError {
    fn into(self) -> JsValue {
        WasmError::to_string(&self).into()
    }
}

#[wasm_bindgen]
impl NBT {
    ///从字节流中读取数据返回[`NBT`]
    pub fn from(bytes: Box<[u8]>) -> IResult<NBT> {
        let mut vec = bytes.to_vec();
        let mut c = Cursor::new(&mut vec);
        Ok(NBT(crate::NBT::read(&mut c)?))
    }

    pub fn named(name: &str) -> IResult<NBT> {
        Ok(NBT(crate::NBT {
            name: name.to_owned(),
            data: Value::Compound(Map::new()),
        }))
    }

    #[wasm_bindgen(constructor)]
    pub fn new(v: INBT) -> WResult<NBT> {
        Ok(NBT(serde_wasm_bindgen::from_value::<crate::NBT>(v.into())?))
    }

    ///返回内部的[`crate::NBT`]值
    #[wasm_bindgen(getter)]
    pub fn value(&self) -> WResult<INBT> {
        Ok(INBT {
            obj: serde_wasm_bindgen::to_value(&self.0)?,
        })
    }

    ///按照 ```path``` 路径设置 ```value``` 值
    pub fn set(&mut self, path: String, value: Option<IValue>) -> WResult<()> {
        if let Value::Compound(m) = &mut self.0.data {
            let path = std::path::Path::new(&path).iter();
            let last = path.clone().last().ok_or(WasmError::InvalidStr)?;
            let mut map: *mut Map<String, Value> = m;

            for p in path {
                let s = p.to_str().ok_or(WasmError::InvalidStr)?;

                if p == last {
                    if let Some(val) = &value {
                        deref_map(map).insert(
                            s.to_string(),
                            serde_wasm_bindgen::from_value::<crate::Value>(val.into())?,
                        );
                    } else {
                        deref_map(map).remove(s);
                    }
                    break;
                }

                match deref_map(map).get_mut(s) {
                    Some(v) => {
                        if let Value::Compound(c) = v {
                            map = c;
                        } else {
                            return Err(WasmError::InvalidPath(format!(
                                "{s} 路径下不是Compound标签"
                            )));
                        }
                    }
                    None => {
                        deref_map(map).insert(s.to_string(), Value::Compound(Map::new()));
                        map = unsafe {
                            &mut *((deref_map(map).to_back(s).unwrap() as *mut Value as usize
                                + 0x08)
                                as *mut Map<String, Value>)
                        };
                    }
                }
            }

            //self.0.header = Header::new(self.bytes(true))
        } else {
            return Err(WasmError::Error(Error::Root(255)));
        }
        Ok(())
    }

    ///返回生成的字节流
    pub fn bytes(&self, bedrock_header: bool) -> IResult<Box<[u8]>> {
        let mut vec = Vec::<u8>::new();
        self.0.write(&mut vec, bedrock_header)?;
        Ok(vec.into_boxed_slice())
    }

    /// 设置内部的[`crate::NBT`]的```name```字段
    #[wasm_bindgen(setter)]
    pub fn set_set_name(&mut self, s: String) {
        self.0.name = s
    }

    /// 获取内部的[`crate::NBT`]的```bedrock_header```
    #[wasm_bindgen(getter)]
    pub fn header(&self) -> IResult<Option<Box<[u8]>>> {
        if let Some(h) = crate::NBT::header(&mut &*self.bytes(true)?)? {
            Ok(Some(Box::new(h)))
        } else {
            Ok(None)
        }
    }

    ///js -> toString() 调用[`Self::fmt`]方法
    #[wasm_bindgen(js_name = toString)]
    pub fn to_str(&self) -> String {
        self.to_string()
    }
}

#[inline(always)]
fn deref_map<'a>(r: *mut Map<String, Value>) -> &'a mut Map<String, Value> {
    unsafe { &mut *r }
}

impl Display for NBT {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#[WASM]\n{}", self.0.to_string())
    }
}
