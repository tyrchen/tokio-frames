use bytes::{Bytes, BytesMut};
use educe::Educe;
use prost::Message;
use std::{marker::PhantomData, pin::Pin};

use super::{Deserializer, Serializer};

#[derive(Educe)]
#[educe(Debug, Default)]
pub struct Protobuf<Item, SinkItem> {
    #[educe(Debug(ignore), Default(expression = "PhantomData"))]
    ghost: PhantomData<(Item, SinkItem)>,
}

pub type SymmetricalProtobuf<T> = Protobuf<T, T>;

impl<Item, SinkItem> Deserializer<Item> for Protobuf<Item, SinkItem>
where
    Item: Message + Default,
{
    type Error = prost::DecodeError;

    fn deserialize(self: Pin<&mut Self>, src: &BytesMut) -> Result<Item, Self::Error> {
        Item::decode(src.as_ref())
    }
}

impl<Item, SinkItem> Serializer<SinkItem> for Protobuf<Item, SinkItem>
where
    SinkItem: Message + Default,
{
    type Error = prost::EncodeError;

    fn serialize(self: Pin<&mut Self>, item: &SinkItem) -> Result<Bytes, Self::Error> {
        let mut buf = BytesMut::with_capacity(256);
        item.encode(&mut buf)?;
        Ok(buf.freeze())
    }
}

// This seems doesn't work - it can be compiled however the returned type is not working for SinkExt or StreamExt
// pub fn framed_transport<T, Item: Message + Default>(
//     socket: T,
// ) -> Framed<Framed<T, Item, Item, LengthDelimitedCodec>, Item, Item, SymmetricalProtobuf<Item>> {
//     let length_delimited = Framed::new(socket, LengthDelimitedCodec::new());
//     SymmetricallyFramed::new(length_delimited, SymmetricalProtobuf::default())
// }

#[cfg(test)]
mod tests {
    use impls::impls;
    use std::fmt::Debug;

    use super::*;

    #[test]
    fn prost_impls() {
        struct Nothing;
        type T = Protobuf<Nothing, Nothing>;

        assert!(impls!(T: Debug));
        assert!(impls!(T: Default));
    }
}
