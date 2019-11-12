use serde::{ser, Serialize};

use error::Error;

pub struct Serializer {
    output: String,
}

#[inline]
pub fn to_string<T>(value: &T) -> Result<String>
where
    T: Serialize,
{
    let mut serializer = Serializer {
        output: String::new(),
    };

    value.serialize(&mut serializer)?;
    Ok(serializer.output)
}

impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = ();

    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, V: bool) -> Result<Self::Item, Self::Error> {
        self.output = if v { "true" } else { "false" }
        Ok(())
    }
}


impl<'a> ser::SerializeMap for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> > Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        key.serialize(&mut **self)?;
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> > Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize
    {
        self.output += "__";
        value.serialize(&mut **self)?;
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.output += "\n";
        Ok(())
    }
}

impl<'a> ser::SerializeStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field(&mut self, key: &'static str, value: &T) -> Result<Self::Ok, Self::Error> {
        key.serialize(&mut **self);
        self.output += "__";
        `
    }


}
