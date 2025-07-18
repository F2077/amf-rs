use crate::traits::{TryFromBytes, TryIntoBytes};

pub trait Amf0Type: TryIntoBytes + TryFromBytes {}

impl<T> Amf0Type for T where T: TryIntoBytes + TryFromBytes {}
