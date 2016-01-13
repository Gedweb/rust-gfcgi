use std::net::TcpListener;

extern crate fcgi;

use fcgi::fcgi::client;

fn main() {

    let host_port: &str = "localhost:4128";

    let listener = match TcpListener::bind(host_port) {
        Ok(result) => result,
        Err(_) => panic!("Can't bind {}", host_port),
    };

    for stream in listener.incoming() {
        
        let mut client = match stream {
            Ok(stream) => client::Stream::new(stream),
            Err(error) => panic!("Connection error {}", error),
        };
        
        for (request_id, request_body) in client.read() {
            
            println!("{:?}", request_body);
            client.write(request_id);
        }
    }
}