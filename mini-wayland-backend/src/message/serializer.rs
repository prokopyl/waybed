use std::marker::PhantomData;

pub struct Serializer<'a> {
    inner: PhantomData<&'a ()>,
}

impl<'a> Serializer<'a> {}
