//!
//! DeserializeBuilder.
//!
//! For deserialization, there are several options that can be modified, such as :
//!
//! - array separator
//! - field separator
//! - parent node (whether we have divergen parent/namespace in the envs or not),
//!   will create parent node (Node::Branch) that points to all detected divergent root node
//!   from envs.
//!

use derive_builder::Builder;
use std::str::pattern::Pattern;

#[derive(Builder, Debug)]
#[builder(setter(into))]
pub struct Options<'a, P>
where
    P: Pattern<'a>,
{
    array_sep: P,
    field_sep: P,
    root: Option<String>,
}

#[cfg(test)]
mod test {
    use super::{Options, OptionsBuilder};

    #[test]
    fn test_options_builder() {
        let r = OptionsBuilder::default()
            .array_sep(',')
            .field_sep('=')
            .root(Some("sample"))
            .build();

        assert!(r.is_ok());
    }
}
