use std::net::TcpListener;

extern crate fcgi;

use fcgi::client;

fn main() {

let listener = TcpListener::bind("localhost:4128").unwrap();

let mut client: client::Client = client::Client::new(listener);

    while let Some(request) = client.next() {

        let mut response = request.reply();
        response.set_status(200).set_header("Content-type", "text/plain").set_body("Hello");

//        client.write(&response);
    }
}

