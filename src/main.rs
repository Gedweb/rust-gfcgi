use std::net::TcpListener;

extern crate fcgi;

use fcgi::client;

fn main() {

    let listener = TcpListener::bind("127.0.0.1:4128").unwrap();
    let client: client::Client = client::Client::new(listener);

    client.init();

    for request in client {
        println!("{:?}", request);
//        request.reply().send();
    }

}

