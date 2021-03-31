use bytes::{Bytes, BytesMut};
use educe::Educe;
use rkyv::{
    archived_value,
    de::{deserializers::AllocDeserializer, Deserializer as RkyvDeserializer},
    ser::{serializers::WriteSerializer, Serializer as RkyvSerializer},
    AlignedVec, Fallible, Unreachable,
};
use std::{marker::PhantomData, pin::Pin};

use super::{Deserializer, Serializer};

#[derive(Educe)]
#[educe(Debug, Default)]
pub struct Archive<Item, SinkItem> {
    #[educe(Debug(ignore), Default(expression = "PhantomData"))]
    ghost: PhantomData<(Item, SinkItem)>,
}

pub type SymmetricalArchive<T> = Archive<T, T>;

impl<Item, SinkItem> Deserializer<Item> for Archive<Item, SinkItem>
where
    Item: rkyv::Archive + rkyv::Deserialize<Item, Unreachable>,
{
    type Error = Unreachable;

    fn deserialize(self: Pin<&mut Self>, src: &BytesMut) -> Result<Item, Self::Error> {
        let archived = unsafe { archived_value::<Item>(src.as_ref(), 0) };
        let mut deserializer = AllocDeserializer;
        let deserialized = archived
            .deserialize(&mut deserializer)
            .expect("failed to deserialize value");
        Ok(deserialized)
    }
}

impl<Item, SinkItem> Serializer<SinkItem> for Archive<Item, SinkItem>
where
    SinkItem: rkyv::Archive + rkyv::Serialize<WriteSerializer<AlignedVec>> + Fallible,
{
    type Error = SinkItem::Error;

    fn serialize(self: Pin<&mut Self>, item: &SinkItem) -> Result<Bytes, Self::Error> {
        let mut ser = WriteSerializer::new(AlignedVec::new());
        let pos = ser.serialize_value(item).unwrap();

        let buf = ser.into_inner();

        Ok(Bytes::from(&buf[pos..]))
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
