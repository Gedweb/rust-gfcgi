use std::net::TcpListener;

use std::fs::File;
use std::io::Read;

extern crate fcgi;

use fcgi::client;

fn main() {

    let listener = TcpListener::bind("localhost:4128").unwrap();

    for stream in listener.incoming() {

        let mut client = client::Stream::new(stream.unwrap());

        while let Some(request) = client.next() {

            println!("{:?}", request);

            let mut response = request.reply();

            let mut buf: Vec<u8> = Vec::new();
            let mut file: File = File::open("/home/gedweb/Dropbox/Pictures/crow/crow.png").unwrap();
            file.read_to_end(&mut buf).unwrap();


            response.set_status(200).set_header("Content-type", "image/png").set_body(&buf);

            client.write(&response);
        }

    }
}
