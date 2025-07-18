use crate::traits::{Marshall, Unmarshall};

pub(crate) trait Amf0Type: Marshall + Unmarshall {}

// blanket implementation
impl<T> Amf0Type for T where T: Marshall + Unmarshall {}
