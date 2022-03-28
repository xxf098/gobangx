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

    pub fn is_number(&self) -> bool {
        self.col_type == ColType::Int || self.col_type == ColType::Float
    }

    pub fn is_no_quote(&self) -> bool {
        self.is_number() || self.col_type == ColType::Boolean
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
#[derive(Debug)]
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

impl<S> PartialEq<S> for Value where S: AsRef<str> {

    fn eq(&self, other: &S) -> bool {
        self.is_null == false && self.data == other.as_ref()
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



pub struct ColumnConstraint {
	pub(crate) definition: String,
	pub(crate) name:       String,
}


pub struct ColumnMeta {
    pub(crate) name: String,
    pub(crate) data_type: String,
    pub(crate) length: i32,
    pub(crate) nullable: bool,
    pub(crate) default: Option<String>,
    pub(crate) is_auto_increment: bool,
    pub(crate) identity_generation: Option<String>,
    pub(crate) check: Option<ColumnConstraint>,
}

impl ColumnMeta {

    pub fn get_data_type(&self) -> String {
        match self.data_type.as_ref() {
            "smallint" => if self.is_auto_increment { "smallserial".to_string() } else { self.data_type.clone() },
            "integer" => if self.is_auto_increment { "serial".to_string() } else { self.data_type.clone() },
            "bigint" => if self.is_auto_increment { "bigserial".to_string() } else { self.data_type.clone() },
            "timestamp without time zone" => return "timestamp".to_string(),
            "time without time zone" => "time".to_string(),
            _ => self.data_type.clone()
        }
    }
}