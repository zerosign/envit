use crate::{
    error::SerializeError,
    serde::{ser, Serialize},
};
use std::{fmt, io, marker::PhantomData};

pub trait StringFormatter {
    fn format(f: &mut io::Write, v: &str) -> fmt::Result;
}

pub struct DefaultStringFormatter;

impl StringFormatter for DefaultStringFormatter {
    #[inline]
    fn format(f: &mut io::Write, v: &str) -> fmt::Result {
        f.write('\"')?;
        f.write(v)?;
        f.write('\"')?;
    }
}

pub trait ArrayFormatter {
    fn begin(f: &mut io::Write) -> fmt::Result;
    fn separate(f: &mut io::Write) -> fmt::Result;
    fn end(f: &mut io::Write) -> fmt::Result;
}

pub struct DefaultArrayFormatter;

impl ArrayFormatter for DefaultArrayFormatter {
    #[inline]
    fn begin(f: &mut io::Write) -> fmt::Result {
        f.write('[')
    }

    #[inline]
    fn separate(f: &mut io::Write) -> fmt::Result {
        f.write(',')
    }

    fn end(f: &mut io::Write) -> fmt::Result {
        f.write(']')
    }
}

pub trait FieldFormatter {
    fn pair_sep(f: &mut io::Write) -> fmt::Result;
    fn field_sep(f: &mut io::Write) -> fmt::Result;
}

pub struct DefaultFieldFormatter;

impl FieldFormatter for DefaultFieldFormatter {
    #[inline]
    fn pair_sep(f: &mut io::Write) -> fmt::Result {
        f.write('=')
    }

    #[inline]
    fn field_sep(f: &mut io::Write) -> fmt::Result {
        f.write("__")
    }
}

pub struct Serializer<'a, A, W, F, S>
where
    W: io::Write + Sized,
    A: ArrayFormatter + Sized,
    F: FieldFormatter + Sized,
    S: StringFormatter + Sized,
{
    output: &'a W,
    _array: PhantomData<A>,
    _field: PhantomData<F>,
    _string: PhantomData<S>,
}

impl<'a, 'ser, A, W, F, S> ser::Serializer for &'ser mut Serializer<'a, A, W, F, S>
where
    W: io::Write + Sized,
    A: ArrayFormatter + Sized,
    F: FieldFormatter + Sized,
    S: StringFormatter + Sized,
{
    type Ok = ();

    type Error = SerializeError<'a>;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    #[inline]
    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        if v {
            self.output.write("true")?
        } else {
            self.output.write("false")?
        }
    }

    #[inline]
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.output.write(&v.to_string())?
    }

    #[inline]
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.output.write(&v.to_string())?
    }

    #[inline]
    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.output.write(&v.to_string())?
    }

    #[inline]
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Self::S::format(&mut self.output, v)?
        // self.output += "\"";
        // // TODO: do we need to ensure string quotation ?
        // self.output += v;
        // self.output += "\"";
        // Ok(())
    }

    #[inline]
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Self::A::begin(&mut self.output)?
    }

    #[inline]
    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(f64::from(v))
    }

    #[inline]
    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(i64::from(v))
    }

    #[inline]
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(i64::from(v))
    }

    #[inline]
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(i64::from(v))
    }

    #[inline]
    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(u64::from(v))
    }

    #[inline]
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(u64::from(v))
    }

    #[inline]
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(u64::from(v))
    }

    #[inline]
    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(&v.to_string())
    }

    #[inline]
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        use serde::ser::SerializeSeq;
        let mut seq = self.serialize_seq(Some(v.len()))?;
        for byte in v {
            seq.serialize_element(byte)?;
        }
        seq.end()
    }

    #[inline]
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }
}

impl<'a, 'ser, A, W, F, S> ser::SerializeSeq for &'a mut Serializer<'a, A, W, F, S>
where
    W: io::Write + Sized,
    A: ArrayFormatter + Sized,
    F: FieldFormatter + Sized,
    S: StringFormatter + Sized,
{
    type Ok = ();
    type Error = SerializeError<'a>;

    fn serialize_element<T>(&mut self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        if !self.output.ends_with('[') {
            self.output += ",";
        }
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        A::end(&mut self.output).map_err(Self::Error::custom)
    }
}

impl<'a, 'ser, A, W, F, S> ser::SerializeMap for &'a mut Serializer<'a, A, W, F, S>
where
    W: io::Write + Sized,
    A: ArrayFormatter + Sized,
    F: FieldFormatter + Sized,
    S: StringFormatter + Sized,
{
    type Ok = ();
    type Error = SerializeError<'a>;

    fn serialize_key<T>(&mut self, key: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Ok(())
    }
}
