///!
///! Most abstract trait for serialization & deserialization purposes.
///!
///!
use std::io;

/// Trait that give a way to format a quoted string.
///
pub trait StringFormatter {
    /// Entry point for formatting the strig.
    ///
    /// NOTE: This method may need to check whether given string is already
    /// being quoted or not.
    ///
    fn format<W>(f: &mut W, v: &str) -> io::Result<()>
    where
        W: io::Write + ?Sized;
}

/// Trait that give a way to format
/// an array/sequence like structure.
///
pub trait ArrayFormatter {
    /// Begin formatting array
    ///
    /// most of the usecase of this function is to
    /// write characters before iterating of the elements.
    ///
    fn begin<W>(f: &mut W) -> io::Result<()>
    where
        W: io::Write + ?Sized;

    /// Write token that separates each element in Array like structure.
    ///
    fn separate<W>(f: &mut W) -> io::Result<()>
    where
        W: io::Write + ?Sized;

    /// Write token after last element array reached.
    ///
    fn end<W>(f: &mut W) -> io::Result<()>
    where
        W: io::Write + ?Sized;
}

/// Formatter for writing fields in pair.
///
pub trait FieldFormatter {
    /// Separator that separate between key & value element.
    ///
    ///
    fn pair_sep<W>(f: &mut W) -> io::Result<()>
    where
        W: io::Write + ?Sized;

    /// Separator that separate between each fields in key.
    ///
    ///
    fn field_sep<W>(f: &mut W) -> io::Result<()>
    where
        W: io::Write + ?Sized;

    /// Separator that separate between each pair
    /// (or after value is written)
    ///
    ///
    fn value_sep<W>(f: &mut W) -> io::Result<()>
    where
        W: io::Write + ?Sized;
}


pub trait Writer<W, A, F, S> where W: io::Write {}
