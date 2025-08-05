use serde::Serialize;
use serde::Serializer;

pub(crate) struct RawBytes<T: AsRef<[u8]>>(pub T);
pub(crate) const CP_RAWBYTES_NEWTYPE_STRUCT: &str = "RawBytes";

impl<T: AsRef<[u8]>> Serialize for RawBytes<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        serializer.serialize_bytes(&self.0.as_ref())
    }
}
