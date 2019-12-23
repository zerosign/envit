///!
///! Notes:
///!
///! [`StringFormatter`] & [`ArrayFormatter`] & [`FieldFormatter`] are
///! static trait, means you don't need to have a real struct implementation
///! since most of the methods doesn't need to have instance of the struct.
///!
use crate::{
    error::SerializeError,
    serde::{ser, Serialize},
};
use std::{io, marker::PhantomData};

pub trait StringFormatter {
    fn format<W>(f: &mut W, v: &str) -> io::Result<()>
    where
        W: io::Write;
}

pub struct DefaultStringFormatter;

impl StringFormatter for DefaultStringFormatter {
    #[inline]
    fn format<W>(f: &mut W, v: &str) -> io::Result<()>
    where
        W: io::Write,
    {
        // // TODO: do we need to ensure string quotation validity ?
        write!(&mut f, "\"{}\"", v)
    }
}

pub trait ArrayFormatter {
    fn begin<W>(f: &mut W) -> io::Result<()>
    where
        W: io::Write;

    fn separate<W>(f: &mut W) -> io::Result<()>
    where
        W: io::Write;

    fn end<W>(f: &mut W) -> io::Result<()>
    where
        W: io::Write;
}

pub struct DefaultArrayFormatter;

impl ArrayFormatter for DefaultArrayFormatter {
    #[inline]
    fn begin<W>(f: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        write!(&mut f, "{}", "[")
    }

    #[inline]
    fn separate<W>(f: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        write!(&mut f, "{}", ',')
    }

    fn end<W>(f: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        write!(&mut f, "{}", "]")
    }
}

pub trait FieldFormatter {
    fn pair_sep<W>(f: &mut W) -> io::Result<()>
    where
        W: io::Write;
    fn field_sep<W>(f: &mut W) -> io::Result<()>
    where
        W: io::Write;

    fn value_sep<W>(f: &mut W) -> io::Result<()>
    where
        W: io::Write;
}

pub struct DefaultFieldFormatter;

impl FieldFormatter for DefaultFieldFormatter {
    #[inline]
    fn pair_sep<W>(f: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        write!(&mut f, "{}", "=")
    }

    #[inline]
    fn field_sep<W>(f: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        write!(&mut f, "{}", "__")
    }

    #[inline]
    fn value_sep<W>(f: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        write!(&mut f, "{}", "\n")
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

pub enum SeqFlow<'a, A, W, F, S>
where
    W: io::Write + Sized,
    A: ArrayFormatter + Sized,
    F: FieldFormatter + Sized,
    S: StringFormatter + Sized,
{
    Initial(&'a mut Serializer<'a, A, W, F, S>),
    Next(&'a mut Serializer<'a, A, W, F, S>),
}

impl<'a, A, W, F, S> ser::Serializer for &'a mut Serializer<'a, A, W, F, S>
where
    W: io::Write + Sized,
    A: ArrayFormatter + Sized,
    F: FieldFormatter + Sized,
    S: StringFormatter + Sized,
{
    type Ok = ();

    type Error = SerializeError<'a>;

    type SerializeSeq = SeqFlow<'a, A, W, F, S>;
    type SerializeTuple = SeqFlow<'a, A, W, F, S>;
    type SerializeTupleStruct = SeqFlow<'a, A, W, F, S>;
    type SerializeTupleVariant = SeqFlow<'a, A, W, F, S>;
    type SerializeMap = SeqFlow<'a, A, W, F, S>;
    type SerializeStruct = SeqFlow<'a, A, W, F, S>;
    type SerializeStructVariant = SeqFlow<'a, A, W, F, S>;

    #[inline]
    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        write!(&mut *self.output, "{}", v.to_string()).map_err(Self::Error::from)
    }

    #[inline]
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        write!(&mut *self.output, "{}", v.to_string()).map_err(Self::Error::from)
    }

    #[inline]
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        write!(&mut *self.output, "{}", v.to_string()).map_err(Self::Error::from)
    }

    #[inline]
    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        write!(&mut *self.output, "{}", &v.to_string()).map_err(Self::Error::from)
    }

    #[inline]
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        S::format(&mut *self.output, v).map_err(Self::Error::from)
    }

    #[inline]
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        A::begin(&mut *self.output)?;
        Ok(SeqFlow::Initial(&mut self))
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
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    #[inline]
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    #[inline]
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    #[inline]
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        write!(&mut self.output, "{}", "null").map_err(Self::Error::from)
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(variant)
    }

    #[inline]
    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    #[inline]
    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        variant.serialize(&mut *self)?;
        F::pair_sep(&mut *self.output)?;
        value.serialize(&mut *self)?;
        F::value_sep(&mut *self.output).map_err(Self::Error::from)
    }
}

impl<'a, A, W, F, S> ser::SerializeSeq for SeqFlow<'a, A, W, F, S>
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
        match self {
            Self::Initial(mut v) => value.serialize(&mut *v),
            Self::Next(mut v) => {
                A::separate(&mut *v.output)?;
                value.serialize(&mut *v)
            }
        }
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self {
            Self::Initial(v) => A::end(&mut *v.output).map_err(Self::Error::from),
            Self::Next(v) => A::end(&mut *v.output).map_err(Self::Error::from),
        }
    }
}

impl<'a, A, W, F, S> ser::SerializeTuple for SeqFlow<'a, A, W, F, S>
where
    W: io::Write + Sized,
    A: ArrayFormatter + Sized,
    F: FieldFormatter + Sized,
    S: StringFormatter + Sized,
{
    type Ok = ();
    type Error = SerializeError<'a>;

    #[inline]
    fn serialize_element<T>(&mut self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        match self {
            Self::Initial(mut v) => value.serialize(&mut *v),
            Self::Next(mut v) => {
                A::separate(&mut *v.output)?;
                value.serialize(&mut *v)
            }
        }
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self {
            Self::Initial(v) => A::end(&mut *v.output).map_err(Self::Error::from),
            Self::Next(v) => A::end(&mut *v.output).map_err(Self::Error::from),
        }
    }
}

impl<'a, A, W, F, S> ser::SerializeTupleStruct for SeqFlow<'a, A, W, F, S>
where
    W: io::Write + Sized,
    A: ArrayFormatter + Sized,
    F: FieldFormatter + Sized,
    S: StringFormatter + Sized,
{
    type Ok = ();
    type Error = SerializeError<'a>;

    fn serialize_field<T>(&mut self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        match self {
            Self::Initial(mut v) => value.serialize(&mut *v),
            Self::Next(mut v) => {
                A::separate(&mut *v.output)?;
                value.serialize(&mut *v)
            }
        }
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self {
            Self::Initial(v) => A::end(&mut *v.output).map_err(Self::Error::from),
            Self::Next(v) => A::end(&mut *v.output).map_err(Self::Error::from),
        }
    }
}

impl<'a, A, W, F, S> ser::SerializeTupleVariant for SeqFlow<'a, A, W, F, S>
where
    W: io::Write + Sized,
    A: ArrayFormatter + Sized,
    F: FieldFormatter + Sized,
    S: StringFormatter + Sized,
{
    type Ok = ();
    type Error = SerializeError<'a>;

    #[inline]
    fn serialize_field<T>(&mut self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        match self {
            Self::Initial(mut v) => value.serialize(&mut *v),
            Self::Next(mut v) => {
                A::separate(&mut *v.output)?;
                value.serialize(&mut *v)
            }
        }
    }
    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self {
            Self::Initial(v) => A::end(&mut *v.output).map_err(Self::Error::from),
            Self::Next(v) => A::end(&mut *v.output).map_err(Self::Error::from),
        }
    }
}

impl<'a, A, W, F, S> ser::SerializeMap for SeqFlow<'a, A, W, F, S>
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
        match self {
            Self::Initial(mut v) => key.serialize(&mut *v),
            Self::Next(mut v) => {
                F::field_sep(&mut *v.output)?;
                key.serialize(&mut *v)
            }
        }
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        match self {
            Self::Initial(mut v) => value.serialize(&mut *v),
            Self::Next(mut v) => {
                F::pair_sep(&mut *v.output)?;
                value.serialize(&mut *v)
            }
        }
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self {
            Self::Initial(mut v) => F::value_sep(&mut *v.output).map_err(Self::Error::from),
            Self::Next(mut v) => F::value_sep(&mut *v.output).map_err(Self::Error::from),
        }
    }
}

impl<'a, A, W, F, S> ser::SerializeStruct for SeqFlow<'a, A, W, F, S>
where
    W: io::Write + Sized,
    A: ArrayFormatter + Sized,
    F: FieldFormatter + Sized,
    S: StringFormatter + Sized,
{
    type Ok = ();
    type Error = SerializeError<'a>;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        match self {
            Self::Initial(mut v) => {
                key.serialize(&mut *v)?;
                F::pair_sep(&mut *v.output)?;
                value.serialize(&mut *v)
            }
            Self::Next(mut v) => {
                F::field_sep(&mut *v.output)?;
                key.serialize(&mut *v)?;
                F::pair_sep(&mut *v.output)?;
                value.serialize(&mut *v)
            }
        }
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self {
            Self::Initial(mut v) => F::value_sep(&mut *v.output).map_err(Self::Error::from),
            Self::Next(mut v) => F::value_sep(&mut *v.output).map_err(Self::Error::from),
        }
    }
}

impl<'a, A, W, F, S> ser::SerializeStructVariant for SeqFlow<'a, A, W, F, S>
where
    W: io::Write + Sized,
    A: ArrayFormatter + Sized,
    F: FieldFormatter + Sized,
    S: StringFormatter + Sized,
{
    type Ok = ();
    type Error = SerializeError<'a>;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        match self {
            Self::Initial(mut v) => {
                key.serialize(&mut *v)?;
                F::pair_sep(&mut *v.output)?;
                value.serialize(&mut *v)
            }
            Self::Next(mut v) => {
                F::field_sep(&mut *v.output)?;
                key.serialize(&mut *v)?;
                F::pair_sep(&mut *v.output)?;
                value.serialize(&mut *v)
            }
        }
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self {
            Self::Initial(mut v) => F::value_sep(&mut *v.output).map_err(Self::Error::from),
            Self::Next(mut v) => F::value_sep(&mut *v.output).map_err(Self::Error::from),
        }
    }
}
