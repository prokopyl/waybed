mod serializer;
pub use serializer::Serializer;

pub trait Message {
    fn serialize(&self, serializer: &mut Serializer<'_>);
    fn data_len(&self) -> usize;
    fn fd_count(&self) -> u8;
}
