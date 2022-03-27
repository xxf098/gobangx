use std::string::ToString;
use std::fmt;
use std::convert::{From};

#[derive(Clone, PartialEq, Debug)]
pub enum ColType {
    VarChar,
    Int,
    Float,
    Boolean,
    Date,
    Json,
    Unknown,
}

#[derive(Debug)]
pub struct Header {
    pub name: String,
    pub col_type: ColType, 
}

impl Header {

    pub fn new(name: String, typ: ColType) -> Self {
        Self { name, col_type: typ }
    }
}


impl Clone for Header {

    fn clone(&self) -> Self {
        Self { name: self.name.clone(), col_type: self.col_type.clone() }
    }
}

impl ToString for Header {

    fn to_string(&self) -> String {
        self.name.clone()
    }
}


impl From<&str> for Header {

    fn from(item: &str) -> Self {
        Self { name: item.to_string(), col_type: ColType::Unknown }
    }
}

// for cell value
pub struct Value {
    pub data: String,
    pub is_null: bool,
}



impl Value {
    pub fn new(v: String) -> Self {
        Self { data: v, is_null: false }
    }
}

impl Clone for Value {

    fn clone(&self) -> Self {
        Self { data: self.data.clone(), is_null: self.is_null }
    }
}

// impl From<String> for Value {

//     fn from(v: String) -> Self {
//         Self::new(v)
//     }
// }

impl<S> From<S> for Value where S: AsRef<str> {

    fn from(v: S) -> Self {
        Self::new(v.as_ref().to_string())
    }
}

// impl ToString for Value {

//     fn to_string(&self) -> String {
//         self.data.clone()
//     }
// }

impl fmt::Display for Value {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.data)
    }
}


impl Default for Value {
    fn default() -> Self {
        Self { data: "NULL".to_string(), is_null: true }
    }
}

