use std::net::TcpListener;

extern crate fcgi;

use fcgi::client::{self, model};

fn main() {

    let listener = TcpListener::bind("127.0.0.1:4128").unwrap();
    let client = client::Client::new(listener);

    client.run();

    for request in client {
//        println!("{:?}", request);
        let mut response = model::Response::new();

        response.set_body("Hello!");
        response.set_status(428);
        request.reply(response);
    }

}

