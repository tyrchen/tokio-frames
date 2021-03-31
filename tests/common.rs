use bytes::Bytes;
use futures::prelude::*;
use prost::Message;
use tokio::net::{TcpListener, TcpStream};
use tokio_frames::protobuf::SymmetricalProtobuf;
use tokio_frames::SymmetricallyFramed;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum Status {
    Ok = 0,
    NotFound = 1,
    InternalError = 2,
}

#[derive(Clone, PartialEq, Message)]
pub struct Hello {
    #[prost(string, tag = "2")]
    pub msg: String,
    #[prost(bytes = "bytes", tag = "3")]
    pub data: Bytes,
}

#[derive(Clone, PartialEq, Message)]
pub struct HelloReply {
    #[prost(enumeration = "Status", tag = "1")]
    pub code: i32,
    #[prost(string, tag = "2")]
    pub msg: String,
    #[prost(bytes = "bytes", tag = "3")]
    pub data: Bytes,
}

#[derive(Clone, PartialEq, Message)]
pub struct Msg {
    #[prost(bytes = "bytes", tag = "1")]
    pub id: Bytes,
    #[prost(oneof = "msg::Data", tags = "2, 3")]
    pub data: Option<msg::Data>,
}

pub mod msg {

    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Data {
        #[prost(message, tag = "2")]
        Hello(super::Hello),
        #[prost(message, tag = "3")]
        HelloReply(super::HelloReply),
    }
}

pub async fn client_send(addr: &str, msg: Msg) -> Msg {
    let socket = TcpStream::connect(addr).await.unwrap();
    let length_delimited = Framed::new(socket, LengthDelimitedCodec::new());
    let mut transport = SymmetricallyFramed::new(length_delimited, SymmetricalProtobuf::default());

    transport.send(msg).await.unwrap();
    transport.next().await.unwrap().unwrap()
}

pub async fn start_server(addr: &str) {
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("Listening on: {:?}", addr);
    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let length_delimited = Framed::new(socket, LengthDelimitedCodec::new());
        let mut transport =
            SymmetricallyFramed::new(length_delimited, SymmetricalProtobuf::<Msg>::default());

        tokio::spawn(async move {
            while let Some(req) = transport.try_next().await.unwrap() {
                println!("GOT: {:?}", req);
                if let msg::Data::Hello(data) = req.data.unwrap() {
                    let reply = Msg {
                        id: req.id,
                        data: Some(msg::Data::HelloReply(HelloReply {
                            code: 0,
                            msg: data.msg,
                            data: data.data,
                        })),
                    };
                    transport.send(reply).await.unwrap();
                }
            }
        });
    }
}
