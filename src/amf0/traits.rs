use crate::traits::{FromBytes, ToBytes};

pub trait Amf0Type: ToBytes + FromBytes {}

impl<T> Amf0Type for T where T: ToBytes + FromBytes {}
