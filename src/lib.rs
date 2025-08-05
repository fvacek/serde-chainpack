pub mod de;
pub mod error;
pub mod ser;
pub mod types;
pub mod cpdatetime;

use serde::{Serialize, ser::Serializer};

pub struct RawBytes<T: AsRef<[u8]>>(pub T);

impl<T: AsRef<[u8]>> Serialize for RawBytes<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        serializer.serialize_bytes(self.0.as_ref())
    }
}