use std::string::ToString;
use std::convert::{From};

#[derive(Clone, PartialEq, Debug)]
pub enum ColType {
    VarChar,
    Int,
    Float,
    Boolean,
    Date,
    Json,
    Null,
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
    val: String,
    is_null: bool,
}
