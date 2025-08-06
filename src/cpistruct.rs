use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct CPIStruct<T>(pub T);
pub(crate) const CP_ISTRUCT_NEWTYPE_STRUCT: &str = "CPIStruct";
