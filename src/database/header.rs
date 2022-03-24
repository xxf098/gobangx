
pub enum ColType {
    VarChar,
    Text,
    Int,
    Null,
    Unknown,
}

pub struct Header {
    pub header: String,
    pub col_type: ColType, 
}