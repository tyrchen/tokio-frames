mod common;

use bytes::Bytes;

use common::*;

#[tokio::test]
async fn client_server_hello_works() {
    let addr = "127.0.0.1:8210";
    tokio::spawn(async move {
        start_server(addr).await;
    });

    tokio::time::sleep(std::time::Duration::from_millis(10)).await;

    let hello = Msg {
        id: Bytes::from_static(b"1234"),
        data: Some(msg::Data::Hello(Hello {
            msg: "hello world!".to_owned(),
            data: Bytes::from_static(b"hello world!"),
        })),
    };
    let reply = client_send(addr, hello).await;
    assert_eq!(reply.id.as_ref(), b"1234");
}
