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

        let a = match stream {
            Ok(n) => n,
            message @ Err(_) => panic!(message),
        };

        let mut client = client::StreamReader::new(&a);

        loop {

            let request = match client.next() {
                Some(r) => r,
                None => break,
            };

            println!("{:?}", request);

            let mut response = request.reply();
            response.status = 503;
            response.body = b"Hello my friend!".to_vec();
            response.header.insert("Content-type".to_string(), "application/json".to_string());

            client.write(&response);
        }

    }
}
