use std::io::Write;
use serde::ser::{self, Serialize};
use crate::error::{Result, Error};
use crate::types;
use byteorder::{LittleEndian, WriteBytesExt};

pub struct Serializer<W> {
    writer: W,
}

impl<W: Write> Serializer<W> {
    pub fn new(writer: W) -> Self {
        Serializer { writer }
    }
}

impl<'a, W: Write> ser::Serializer for &'a mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        self.writer.write_u8(if v { types::CP_TRUE } else { types::CP_FALSE })?;
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok> {
        if v >= 0 && v < 64 {
            self.writer.write_u8(v as u8)?;
        } else {
            self.writer.write_u8(types::CP_INT)?;
            self.writer.write_i64::<LittleEndian>(v)?;
        }
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
        self.serialize_u64(v as u64)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
        self.serialize_u64(v as u64)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok> {
        self.serialize_u64(v as u64)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok> {
        if v < 64 {
            self.writer.write_u8(v as u8)?;
        } else {
            self.writer.write_u8(types::CP_UINT)?;
            let bits = 64 - v.leading_zeros();
            if bits <= 7 {
                self.writer.write_u8(v as u8)?;
            } else if bits <= 14 {
                self.writer.write_u8(0x80 | (v >> 8) as u8)?;
                self.writer.write_u8((v & 0xFF) as u8)?;
            } else if bits <= 21 {
                self.writer.write_u8(0xC0 | (v >> 16) as u8)?;
                self.writer.write_u8(((v >> 8) & 0xFF) as u8)?;
                self.writer.write_u8((v & 0xFF) as u8)?;
            } else if bits <= 28 {
                self.writer.write_u8(0xE0 | (v >> 24) as u8)?;
                self.writer.write_u8(((v >> 16) & 0xFF) as u8)?;
                self.writer.write_u8(((v >> 8) & 0xFF) as u8)?;
                self.writer.write_u8((v & 0xFF) as u8)?;
            } else {
                let num_bytes = (bits as usize + 7) / 8;
                self.writer.write_u8(0xF0 | ((num_bytes - 4) as u8))?;
                self.writer.write_all(&v.to_be_bytes()[8 - num_bytes..])?;
            }
        }
        Ok(())
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok> {
        Err(Error::UnsupportedType)
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok> {
        self.writer.write_u8(types::CP_DOUBLE)?;
        self.writer.write_f32::<LittleEndian>(v)?;
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        self.writer.write_u8(types::CP_STRING)?;
        self.writer.write_all(v.as_bytes())?;
        self.writer.write_u8(0)?;
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
        self.writer.write_u8(types::CP_BLOB)?;
        self.writer.write_u64::<LittleEndian>(v.len() as u64)?;
        self.writer.write_all(v)?;
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        self.serialize_unit()
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        self.writer.write_u8(types::CP_TERM)?;
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        self.writer.write_u8(types::CP_MAP)?;
        variant.serialize(&mut *self)?;
        value.serialize(&mut *self)?;
        self.writer.write_u8(types::CP_TERM)?;
        Ok(())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        self.writer.write_u8(types::CP_LIST)?;
        Ok(self)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.writer.write_u8(types::CP_MAP)?;
        variant.serialize(&mut *self)?;
        self.writer.write_u8(types::CP_LIST)?;
        Ok(self)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        self.writer.write_u8(types::CP_MAP)?;
        Ok(self)
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.writer.write_u8(types::CP_MAP)?;
        variant.serialize(&mut *self)?;
        self.writer.write_u8(types::CP_MAP)?;
        Ok(self)
    }
}

impl<'a, W: Write> ser::SerializeSeq for &'a mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok> {
        self.writer.write_u8(types::CP_TERM)?;
        Ok(())
    }
}

impl<'a, W: Write> ser::SerializeTuple for &'a mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok> {
        self.writer.write_u8(types::CP_TERM)?;
        Ok(())
    }
}

impl<'a, W: Write> ser::SerializeTupleStruct for &'a mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok> {
        self.writer.write_u8(types::CP_TERM)?;
        Ok(())
    }
}

impl<'a, W: Write> ser::SerializeTupleVariant for &'a mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok> {
        self.writer.write_u8(types::CP_TERM)?;
        self.writer.write_u8(types::CP_TERM)?;
        Ok(())
    }
}

impl<'a, W: Write> ser::SerializeMap for &'a mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<()>
    where
        T: Serialize,
    {
        key.serialize(&mut **self)
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok> {
        self.writer.write_u8(types::CP_TERM)?;
        Ok(())
    }
}

impl<'a, W: Write> ser::SerializeStruct for &'a mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        key.serialize(&mut **self)?;
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok> {
        self.writer.write_u8(types::CP_TERM)?;
        Ok(())
    }
}

impl<'a, W: Write> ser::SerializeStructVariant for &'a mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        key.serialize(&mut **self)?;
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok> {
        self.writer.write_u8(types::CP_TERM)?;
        self.writer.write_u8(types::CP_TERM)?;
        Ok(())
    }
}
