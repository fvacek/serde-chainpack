use std::io::Write;
use serde::ser::{self, Serialize};
use crate::error::{Result, Error};
use crate::types;
use byteorder::{LittleEndian, WriteBytesExt};

pub fn to_vec<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    let mut writer = Vec::new();
    let mut serializer = Serializer::new(&mut writer);
    value.serialize(&mut serializer)?;
    Ok(writer)
}

struct RawBytesSerializer<'a, W: Write> {
    pub(crate) ser: &'a mut Serializer<W>,
}

impl<'a, W: Write> ser::Serializer for RawBytesSerializer<'a, W> {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = ser::Impossible<(), Error>;
    type SerializeTuple = ser::Impossible<(), Error>;
    type SerializeTupleStruct = ser::Impossible<(), Error>;
    type SerializeTupleVariant = ser::Impossible<(), Error>;
    type SerializeMap = ser::Impossible<(), Error>;
    type SerializeStruct = ser::Impossible<(), Error>;
    type SerializeStructVariant = ser::Impossible<(), Error>;

    fn serialize_newtype_struct<T: ?Sized>(self, _name: &'static str, value: &T) -> Result<Self::Ok> where T: Serialize {
        value.serialize(self)
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
        self.ser.writer.write_all(v).map_err(Error::from)
    }

    fn serialize_bool(self, _v: bool) -> Result<Self::Ok> { Err(Error::UnsupportedType) }
    fn serialize_i8(self, _v: i8) -> Result<Self::Ok> { Err(Error::UnsupportedType) }
    fn serialize_i16(self, _v: i16) -> Result<Self::Ok> { Err(Error::UnsupportedType) }
    fn serialize_i32(self, _v: i32) -> Result<Self::Ok> { Err(Error::UnsupportedType) }
    fn serialize_i64(self, v: i64) -> Result<Self::Ok> {
        serialize_raw_i64(&mut self.ser.writer, v).map_err(Error::from)
    }
    fn serialize_u8(self, _v: u8) -> Result<Self::Ok> { Err(Error::UnsupportedType) }
    fn serialize_u16(self, _v: u16) -> Result<Self::Ok> { Err(Error::UnsupportedType) }
    fn serialize_u32(self, _v: u32) -> Result<Self::Ok> { Err(Error::UnsupportedType) }
    fn serialize_u64(self, v: u64) -> Result<Self::Ok> {
        serialize_raw_u64(&mut self.ser.writer, v).map_err(Error::from)
    }
    fn serialize_f32(self, _v: f32) -> Result<Self::Ok> { Err(Error::UnsupportedType) }
    fn serialize_f64(self, _v: f64) -> Result<Self::Ok> { Err(Error::UnsupportedType) }
    fn serialize_char(self, _v: char) -> Result<Self::Ok> { Err(Error::UnsupportedType) }
    fn serialize_str(self, _v: &str) -> Result<Self::Ok> { Err(Error::UnsupportedType) }
    fn serialize_none(self) -> Result<Self::Ok> { Err(Error::UnsupportedType) }
    fn serialize_some<T: ?Sized>(self, _value: &T) -> Result<Self::Ok> where T: Serialize { Err(Error::UnsupportedType) }
    fn serialize_unit(self) -> Result<Self::Ok> { Err(Error::UnsupportedType) }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> { Err(Error::UnsupportedType) }
    fn serialize_unit_variant(self, _name: &'static str, _variant_index: u32, _variant: &'static str) -> Result<Self::Ok> { Err(Error::UnsupportedType) }

    fn serialize_newtype_variant<T: ?Sized>(self, _name: &'static str, _variant_index: u32, _variant: &'static str, _value: &T) -> Result<Self::Ok> where T: Serialize { Err(Error::UnsupportedType) }
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> { Err(Error::UnsupportedType) }
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> { Err(Error::UnsupportedType) }
    fn serialize_tuple_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeTupleStruct> { Err(Error::UnsupportedType) }
    fn serialize_tuple_variant(self, _name: &'static str, _variant_index: u32, _variant: &'static str, _len: usize) -> Result<Self::SerializeTupleVariant> { Err(Error::UnsupportedType) }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> { Err(Error::UnsupportedType) }
    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> { Err(Error::UnsupportedType) }
    fn serialize_struct_variant( self, _name: &'static str, _variant_index: u32, _variant: &'static str, _len: usize) -> Result<Self::SerializeStructVariant> { Err(Error::UnsupportedType) }
}

fn serialize_raw_i64<W: Write>(writer: &mut W, v: i64) -> Result<()> {
    let uv = if v < 0 { -v } else { v };
    let bits = 64 - uv.leading_zeros() + 1;
    if bits <= 7 {
        let mut b = uv as u8;
        if v != uv { b |= 0b0100_0000 }
        writer.write_u8(b)?;
    } else if bits <= 14 {
        let mut b = 0b1000_0000 | (uv >> 8) as u8;
        if v != uv { b |= 0b0010_0000 }
        writer.write_u8(b)?;
        writer.write_u8((uv & 0xFF) as u8)?;
    } else if bits <= 21 {
        let mut b = 0b1100_0000 | (uv >> 16) as u8;
        if v != uv { b |= 0b0001_0000 }
        writer.write_u8(b)?;
        writer.write_u8(((uv >> 8) & 0xFF) as u8)?;
        writer.write_u8((uv & 0xFF) as u8)?;
    } else if bits <= 28 {
        let mut b = 0b1110_0000 | (uv >> 24) as u8;
        if v != uv { b |= 0b0000_1000 }
        writer.write_u8(b)?;
        writer.write_u8(((uv >> 16) & 0xFF) as u8)?;
        writer.write_u8(((uv >> 8) & 0xFF) as u8)?;
        writer.write_u8((uv & 0xFF) as u8)?;
    } else {
        let num_bytes = (bits as usize + 7) / 8;
        writer.write_u8(0xF0 | ((num_bytes - 4) as u8))?;
        let bytes = uv.to_be_bytes();
        let mut b = bytes[8 - num_bytes];
        if v != uv { b |= 0b1000_0000 }
        writer.write_u8(b)?;
        let bytes = &bytes[8 - num_bytes + 1..];
        writer.write_all(bytes)?;
    }
    Ok(())
}

fn serialize_raw_u64<W: Write>(writer: &mut W, v: u64) -> Result<()> {
    let bits = 64 - v.leading_zeros();
    if bits <= 7 {
        writer.write_u8(v as u8)?;
    } else if bits <= 14 {
        writer.write_u8(0x80 | (v >> 8) as u8)?;
        writer.write_u8((v & 0xFF) as u8)?;
    } else if bits <= 21 {
        writer.write_u8(0xC0 | (v >> 16) as u8)?;
        writer.write_u8(((v >> 8) & 0xFF) as u8)?;
        writer.write_u8((v & 0xFF) as u8)?;
    } else if bits <= 28 {
        writer.write_u8(0xE0 | (v >> 24) as u8)?;
        writer.write_u8(((v >> 16) & 0xFF) as u8)?;
        writer.write_u8(((v >> 8) & 0xFF) as u8)?;
        writer.write_u8((v & 0xFF) as u8)?;
    }
    else {
        let num_bytes = (bits as usize + 7) / 8;
        writer.write_u8(0xF0 | ((num_bytes - 4) as u8))?;
        writer.write_all(&v.to_be_bytes()[8 - num_bytes..])?;
    }
    Ok(())
}

pub struct Serializer<W> {
    pub(crate) writer: W,
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
            self.writer.write_u8(0x40 + v as u8)?;
        } else {
            self.writer.write_u8(types::CP_INT)?;
            serialize_raw_i64(&mut self.writer, v)?;
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
            serialize_raw_u64(&mut self.writer, v)?;
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
        let bytes = v.as_bytes();
        let len = bytes.len() as u64;
        self.serialize_u64(len)?;
        self.writer.write_all(bytes)?;
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
        self.writer.write_u8(types::CP_BLOB)?;
        let len = v.len() as u64;
        self.serialize_u64(len)?;
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
        self.writer.write_u8(types::CP_NULL)?;
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
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        if name == crate::cpdatetime::CP_DATETIME_NEWTYPE_STRUCT {
            self.writer.write_u8(types::CP_DATETIME)?;
            let rbs = RawBytesSerializer{ ser: self };
            return value.serialize(rbs);
        }
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
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.writer.write_u8(types::CP_LIST)?;
        Ok(self)
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
