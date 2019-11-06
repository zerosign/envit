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

// #[derive(Builder, Debug)]
// #[builder(setter(into))]
// pub struct Options<'a, P>
// where
//     P: Pattern<'a>,
// {
//     array_sep: P,
//     field_sep: P,
//     parent: Option<String>,
// }

// impl <'a> Options<'a, P> where P: Pattern<'a> {
//     #[inline]
//     pub fn array_sep(&self) -> P {
//         self.array_sep
//     }

//     #[inline]
//     pub fn field_sep(&self) -> P {
//         self.field_sep
//     }

//     #{inline]
//     pub fn parent(&self) -> Option<String> {
//         self.parent
//     }
// }

// impl<'a> Default for Options<'a> {
//     fn default() -> Self {
//         Options {
//             array_sep: ";",
//             field_sep: "__",
//             parent: None,
//         }
//     }
// }
