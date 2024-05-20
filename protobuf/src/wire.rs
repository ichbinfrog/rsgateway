pub trait Serializer {
    fn serializer(v: &mut Vec<u8>);
}

impl Serializer for u64 {
    fn serializer(v: &mut Vec<u8>) {}
}
