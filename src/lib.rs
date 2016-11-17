#![allow(dead_code)]
//! This crate provieds FastCGI client with supporting multithreaded socket listener and HTTP-instances multiplexed onto a single connection.

// object
mod fastcgi;
mod http;

pub use http::{Request, Response};

// Data struct
use std::collections::HashMap;
use std::iter::Iterator;

// io
use std::net::{TcpListener, TcpStream, ToSocketAddrs};

// Thread
use std::thread;


#[derive(PartialEq)]
enum ParseStatus {
    Begin,
    Progress,
    End,
}

pub struct Client
{
    listener: TcpListener,
}

impl Client
{
    pub fn new<A: ToSocketAddrs>(addr: A) -> Self
    {
        Client {
            listener: TcpListener::bind(addr).expect("Bind address"),
        }
    }

    pub fn run<T: Handler>(&self)
    {
        let listener = self.listener.try_clone().expect("Clone listener");
        let handler = T::new();

        thread::spawn(move || {

            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        let reader = StreamReader::new(&stream);
                        for mut request in reader {

                            // call handler
                            let response = handler.process(&mut request);

                            // drop not readed data
//                            if !reader.request.get(&id).unwrap().has_readed() {
//                                let mut drop = [0u8; 32];
//                                while match reader.read(&mut drop) {
//                                    Ok(32) => true,
//                                    Ok(_) => false,
//                                    Err(e) => panic!("{}", e),
//                                } {
//                                    drop = [0u8; 32];
//                                }
//                            }

                            // let response
                            match response {
                                Some(_) => (),
                                None => {
                                    http::Response::new(request.get_id());
                                },
                            }
                        }
                    }
                    Err(e) => panic!("{}", e),
                }
            }
        });

        thread::park();
    }
}

pub struct StreamReader<'s>
{
    status: ParseStatus,
    request: HashMap<u16, http::Request<'s>>,
    stream: &'s TcpStream,
}

impl<'s> StreamReader<'s>
{
    fn new(stream: &'s TcpStream) -> Self
    {
        StreamReader {
            status: ParseStatus::Begin,
            request: HashMap::new(),
            stream: stream,
        }
    }
}

impl<'s> Iterator for StreamReader<'s>
{
    type Item = http::Request<'s>;

    fn next(&mut self) -> Option<Self::Item>
    {
        while !self.request.is_empty() || self.status == ParseStatus::Begin {
            let h = http::Request::read_header(self.stream);
            let body = http::Request::read_body(self.stream, h.content_length as usize);

            self.request.entry(h.request_id)
                .or_insert(http::Request::new(self.stream));

            if h.content_length == 0 && (h.type_ == fastcgi::STDIN || h.type_ == fastcgi::DATA) {
                return self.request.remove(&h.request_id);
                self.status = ParseStatus::Progress;
            }

            self.request
                .get_mut(&h.request_id)
                .expect("HTTP request")
                .parse_record(h, body);
        }


        None
    }
}

pub trait Handler: Send + Clone + 'static
{
    fn new() -> Self;

    fn process(&self, &mut Request) -> Option<Response>;
}





