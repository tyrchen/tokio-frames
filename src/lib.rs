//! This crate provides the utilities neended to easily implement a tokio
//! transport using prost for serialization and deserialization of frame
//! values.
//!
//! # Introduction
//!
//! TBD

// pub mod archive;

#[cfg(feature = "protobuf")]
pub mod protobuf;

use bytes::{Bytes, BytesMut};
use futures::{ready, Sink, Stream, TryStream};
use pin_project::pin_project;
use std::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

pub trait Serializer<T> {
    type Error;
    fn serialize(self: Pin<&mut Self>, item: &T) -> Result<Bytes, Self::Error>;
}

pub trait Deserializer<T> {
    type Error;
    fn deserialize(self: Pin<&mut Self>, src: &BytesMut) -> Result<T, Self::Error>;
}

#[pin_project]
#[derive(Debug)]
pub struct Framed<Transport, Item, SinkItem, Codec> {
    #[pin]
    inner: Transport,
    #[pin]
    codec: Codec,
    item: PhantomData<(Item, SinkItem)>,
}

impl<Transport, Item, SinkItem, Codec> Framed<Transport, Item, SinkItem, Codec> {
    /// create a new `Framed` with the given transport and codec
    pub fn new(inner: Transport, codec: Codec) -> Self {
        Self {
            inner,
            codec,
            item: PhantomData,
        }
    }

    /// returns a reference to the underlying transport wrapped by `Framed`
    pub fn get_ref(&self) -> &Transport {
        &self.inner
    }

    /// returns a mutable reference to the underlying transport wrapped by `Framed`
    pub fn get_mut(&mut self) -> &mut Transport {
        &mut self.inner
    }

    /// consumes the `Framed`, returning its underlying transport
    pub fn into_inner(self) -> Transport {
        self.inner
    }
}

impl<Transport, Item, SinkItem, Codec> Stream for Framed<Transport, Item, SinkItem, Codec>
where
    Transport: TryStream<Ok = BytesMut>,
    Transport::Error: From<Codec::Error>,
    BytesMut: From<Transport::Ok>,
    Codec: Deserializer<Item>,
{
    type Item = Result<Item, Transport::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match ready!(self.as_mut().project().inner.try_poll_next(cx)) {
            Some(bytes) => Poll::Ready(Some(Ok(self
                .as_mut()
                .project()
                .codec
                .deserialize(&bytes?)?))),
            None => Poll::Ready(None),
        }
    }
}

impl<Transport, Item, SinkItem, Codec> Sink<SinkItem> for Framed<Transport, Item, SinkItem, Codec>
where
    Transport: Sink<Bytes>,
    Codec: Serializer<SinkItem>,
    Codec::Error: Into<Transport::Error>,
{
    type Error = Transport::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().inner.poll_ready(cx)
    }

    fn start_send(mut self: Pin<&mut Self>, item: SinkItem) -> Result<(), Self::Error> {
        let res = self.as_mut().project().codec.serialize(&item);
        let bytes = res.map_err(Into::into)?;
        self.as_mut().project().inner.start_send(bytes)?;

        Ok(())
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().inner.poll_flush(cx)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        ready!(self.as_mut().poll_flush(cx))?;
        self.project().inner.poll_close(cx)
    }
}

pub type SymmetricallyFramed<Transport, Value, Codec> = Framed<Transport, Value, Value, Codec>;
