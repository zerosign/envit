///!
///! [`StringFormatter`] & [`ArrayFormatter`] & [`FieldFormatter`] are
///! static trait, means you don't need to have a real struct implementation
///! since most of the methods doesn't need to have instance of the struct.
///!
///! When serializing a data structure into envs, you need to realize that :
///! - env value could be another field, thus we need to know whether
///!   current node is leaf/value node
///! - any `Serializer::serialize_*` (value serialization fn) can be called in any cases,
///!   not only when serializing a value. It could be called when serializing key in env pair.
///! - Serializing sibling node requires us to keep track the parent nodes (allocations).
///!
///! Since in `serde`, all serialization also derive from Serializer function including
///! field/key serializer or value serializer inside variant. So, we need to create
///! flag mechanism to check whether current context is come from `MapFlow` or `SeqFlow`.
///!
use crate::{
    error::SerializeError,
    serde::{ser, Serialize},
    types::{ArrayFormatter, FieldFormatter, StringFormatter},
};

use std::{io, marker::PhantomData};

pub struct DefaultStringFormatter;

impl StringFormatter for DefaultStringFormatter {
    #[inline]
    fn format<W>(f: &mut W, v: &str) -> io::Result<()>
    where
        W: io::Write + ?Sized,
    {
        // // TODO: do we need to ensure string quotation validity ?
        write!(f, "\"{}\"", v)
    }
}

/// Type that implement `ArrayFormatter`.
///
/// it uses :
/// - '[' for `ArrayFormatter::begin`
/// - ',' for `ArrayFormatter::separate`
/// - ']' for `ArrayFormatter::end`
///
pub struct DefaultArrayFormatter;

impl ArrayFormatter for DefaultArrayFormatter {
    #[inline]
    fn begin<W>(f: &mut W) -> io::Result<()>
    where
        W: io::Write + ?Sized,
    {
        write!(f, "{}", "[")
    }

    #[inline]
    fn separate<W>(f: &mut W) -> io::Result<()>
    where
        W: io::Write + ?Sized,
    {
        write!(f, "{}", ',')
    }

    fn end<W>(f: &mut W) -> io::Result<()>
    where
        W: io::Write + ?Sized,
    {
        write!(f, "{}", "]")
    }
}

pub struct DefaultFieldFormatter;

impl FieldFormatter for DefaultFieldFormatter {
    #[inline]
    fn pair_sep<W>(f: &mut W) -> io::Result<()>
    where
        W: io::Write + ?Sized,
    {
        write!(f, "{}", "=")
    }

    #[inline]
    fn field_sep<W>(f: &mut W) -> io::Result<()>
    where
        W: io::Write + ?Sized,
    {
        write!(f, "{}", "__")
    }

    #[inline]
    fn value_sep<W>(f: &mut W) -> io::Result<()>
    where
        W: io::Write + ?Sized,
    {
        write!(f, "{}", "\n")
    }
}

/// Type that abstract how data structure being serialized.
///
/// It implements `ser::Serializer`.
///
/// Any calls to serialize_* after calling non_value() and set_value()
/// will be not print a pair_sep.
///
pub struct Serializer<'a, A, W, F, S>
where
    W: io::Write + Sized,
    A: ArrayFormatter + Sized,
    F: FieldFormatter + Sized,
    S: StringFormatter + Sized,
{
    // INFO: we need to use custom Writer in here since
    //       we need to be able to replay parents fields serialization
    //       for serialize the next sibling of current node.
    //       Thus, checking current node is leaf/value or not, is important.
    //
    output: W,
    flag_value: bool,
    // TODO: I need to store fields traversal history (as a Stack) in here
    //       so that I can print (duplicately) parent node
    stack: Vec<&'a str>,
    _array: PhantomData<A>,
    _field: PhantomData<F>,
    _string: PhantomData<S>,
}



impl<'a, A, W, F, S> Serializer<'a, A, W, F, S>
where
    W: io::Write + Sized,
    A: ArrayFormatter + Sized,
    F: FieldFormatter + Sized,
    S: StringFormatter + Sized,
{
    #[inline]
    pub(crate) fn set_value(&mut self) {
        self.flag_value = true;
    }

    #[inline]
    pub(crate) fn non_value(&mut self) {
        self.flag_value = false;
    }

    #[inline]
    pub(crate) fn render_pair_sep(&mut self) {
        if self.is_value() {
            F::pair_sep(&mut self.output);
            self.non_value();
        }
    }

    /// Guards to check whether current node is value or not.
    ///
    /// This useful when traversing using SeqFlow or MapFlow,
    /// so that in case Serialize call method that render
    /// Serializer::serialize_* that directly render value,
    /// we could check whether the value is a leaf or a branch.
    /// If it's a branch then don't render pair_sep, otherwise
    /// render the pair_sep.
    ///
    #[inline]
    pub(crate) fn is_value(&self) -> bool {
        self.flag_value
    }
}

///
/// Derived State to be used in figuring out state inside
/// the loop of Serialize* when iterating over its element.
///
#[derive(Clone, Copy)]
pub(crate) enum State {
    Initial,
    Next,
}

/// Flow that only do 1 field sequential iteration.
///
pub struct SeqFlow<'a, A, W, F, S>
where
    W: io::Write + Sized,
    A: ArrayFormatter + Sized,
    F: FieldFormatter + Sized,
    S: StringFormatter + Sized,
{
    ser: &'a mut Serializer<'a, A, W, F, S>,
    state: State,
}

/// Flow that supports key & value sequential iteration.
///
pub struct MapFlow<'a, A, W, F, S>
where
    W: io::Write + Sized,
    A: ArrayFormatter + Sized,
    F: FieldFormatter + Sized,
    S: StringFormatter + Sized,
{
    ser: &'a mut Serializer<'a, A, W, F, S>,
    key: State,
    value: State,
}

impl<'a, A, W, F, S> MapFlow<'a, A, W, F, S>
where
    W: io::Write + Sized,
    A: ArrayFormatter + Sized,
    F: FieldFormatter + Sized,
    S: StringFormatter + Sized,
{
    #[inline]
    pub fn initial(ser: &'a mut Serializer<'a, A, W, F, S>) -> Self {
        Self {
            ser: ser,
            key: State::Initial,
            value: State::Initial,
        }
    }
}

impl<'a, A, W, F, S> SeqFlow<'a, A, W, F, S>
where
    W: io::Write + Sized,
    A: ArrayFormatter + Sized,
    F: FieldFormatter + Sized,
    S: StringFormatter + Sized,
{
    #[inline]
    pub fn initial(ser: &'a mut Serializer<'a, A, W, F, S>) -> Self {
        Self {
            ser: ser,
            state: State::Initial,
        }
    }

    #[inline]
    pub fn set_initial(&mut self) {
        self.state = State::Initial;
    }

    #[inline]
    pub fn set_next(&mut self) {
        self.state = State::Next;
    }
}

impl<'a, A, W, F, S> ser::Serializer for &'a mut Serializer<'a, A, W, F, S>
where
    W: io::Write + Sized,
    A: ArrayFormatter + Sized,
    F: FieldFormatter + Sized,
    S: StringFormatter + Sized,
{
    type Ok = ();

    type Error = SerializeError;

    type SerializeSeq = SeqFlow<'a, A, W, F, S>;
    type SerializeTuple = SeqFlow<'a, A, W, F, S>;
    type SerializeTupleStruct = SeqFlow<'a, A, W, F, S>;
    type SerializeTupleVariant = SeqFlow<'a, A, W, F, S>;
    type SerializeMap = MapFlow<'a, A, W, F, S>;
    type SerializeStruct = SeqFlow<'a, A, W, F, S>;
    type SerializeStructVariant = SeqFlow<'a, A, W, F, S>;

    #[inline]
    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.render_pair_sep();
        write!(self.output, "{}", v.to_string()).map_err(Self::Error::from)
    }

    #[inline]
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.render_pair_sep();
        write!(self.output, "{}", v.to_string()).map_err(Self::Error::from)
    }

    #[inline]
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.render_pair_sep();
        write!(self.output, "{}", v.to_string()).map_err(Self::Error::from)
    }

    #[inline]
    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.render_pair_sep();
        write!(self.output, "{}", &v.to_string()).map_err(Self::Error::from)
    }

    #[inline]
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.render_pair_sep();
        S::format(&mut self.output, v).map_err(Self::Error::from)
    }

    #[inline]
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        self.render_pair_sep();
        A::begin(&mut self.output)?;
        Ok(SeqFlow::initial(self))
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
        self.render_pair_sep();
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
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.render_pair_sep();
        write!(&mut self.output, "{}", "null").map_err(Self::Error::from)
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
        // INFO: inside the closure non_value and set_value,
        //       all calls to serialize_* will not print separator.
        self.non_value();
        self.serialize_str(variant)?;
        self.set_value();
        // variant.serialize(&mut *self)
        value.serialize(self)?;
        F::value_sep(&mut self.output).map_err(Self::Error::from)
    }

    #[inline]
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        variant.serialize(self)?;
        Ok(SeqFlow::initial(self))
    }

    #[inline]
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(MapFlow::initial(self))
    }

    #[inline]
    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        // this only works because of SerializeStruct
        // has the same flow as SerializeSeq since
        // it process both (key, value) tuple at once
        self.serialize_seq(Some(len))
    }

    #[inline]
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        variant.serialize(&mut *self)?;
        // this only works because of SerializeStruct
        // has the same flow as SerializeSeq since
        // it process both (key, value) tuple at once
        self.serialize_seq(None)
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
    type Error = SerializeError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {

        match self.state {
            State::Initial => {
                // since we only change the state,
                // it's okay to reorder this like this.
                self.state = State::Next;

                self.ser.toggle_value();
                value.serialize(self.ser)
            }
            State::Next => {
                A::separate(&mut self.ser.output)?;
                value.serialize(self.ser)
            }
            _ => Ok(()),
        }
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self.state {
            // `self.state` shouldn't never get a `State::Initial` in here
            // since the state always gonna be altered in `Self::serialize_element` function call
            State::Initial => Err(Self::Error::StateError),
            _ => A::end(&mut self.ser.output).map_err(Self::Error::from),
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
    type Error = SerializeError;

    #[inline]
    fn serialize_element<T>(&mut self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeSeq::end(self)
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
    type Error = SerializeError;

    #[inline]
    fn serialize_field<T>(&mut self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeSeq::end(self)
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
    type Error = SerializeError;

    #[inline]
    fn serialize_field<T>(&mut self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }
    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeSeq::end(self)
    }
}

impl<'a, A, W, F, S> ser::SerializeMap for MapFlow<'a, A, W, F, S>
where
    W: io::Write + Sized,
    A: ArrayFormatter + Sized,
    F: FieldFormatter + Sized,
    S: StringFormatter + Sized,
{
    type Ok = ();
    type Error = SerializeError;

    fn serialize_key<T>(&mut self, key: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        match self.key {
            State::Initial => {
                // this also okay to be reordered since
                // we only change state.
                self.key = State::Next;
                key.serialize(self.ser)
            }
            State::Next => {
                F::field_sep(&mut self.ser.output)?;
                key.serialize(self.ser)
            }
        }
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        match self.value {
            State::Initial => {
                // this also okay to be reordered since
                // we only change state.
                self.value = State::Next;
                value.serialize(self.ser)
            }
            State::Next => value.serialize(self.ser),
        }
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        match (self.key, self.value) {
            (State::Initial, _) => Err(Self::Error::StateError),
            (_, State::Initial) => Err(Self::Error::StateError),
            (_, _) => F::value_sep(&mut self.ser.output).map_err(Self::Error::from),
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
    type Error = SerializeError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {

        match self.state {
            State::Initial => {
                // this also okay to be reordered since
                // we only change state.
                self.state = State::Next;
                key.serialize(self.ser)?;
                value.serialize(self.ser)
            }
            State::Next => {
                F::field_sep(&mut self.ser.output)?;
                key.serialize(self.ser)?;
                F::pair_sep(&mut self.ser.output)?;
                value.serialize(self.ser)
            }
        }
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
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
    type Error = SerializeError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeStruct::serialize_field(self, key, value)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeStruct::end(self)
    }
}
