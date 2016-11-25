#![allow(dead_code)]
//! This crate provieds FastCGI client with supporting multithreaded socket listener and HTTP-instances multiplexed onto a single connection.

// object
mod fastcgi;
mod http;

pub use http::{Request, Response};

// Data struct
use std::collections::HashMap;
use std::iter::Iterator;

// net / io
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::io::Write;

// Thread
use std::thread;

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
                        let reader = StreamSyntax::new(&stream);
                        for mut pair in reader {
                            // call handler
                            handler.process(&mut pair);
                            pair.response().flush().unwrap();
                        }
                    }
                    Err(e) => panic!("{}", e),
                }
            }
        });

        thread::park();
    }
}

pub struct HttpPair<'s>(http::Request<'s>, http::Response<'s>);

impl<'s> HttpPair<'s>
{
    pub fn request(&mut self) -> &mut http::Request<'s>
    {
        &mut self.0
    }

    pub fn response(&mut self) -> &mut http::Response<'s>
    {
        &mut self.1
    }
}

pub struct StreamSyntax<'s>
{
    born: bool,
    pair: HashMap<u16, HttpPair<'s>>,
    stream: &'s TcpStream,
}

impl<'s> StreamSyntax<'s>
{
    fn new(stream: &'s TcpStream) -> Self
    {
        StreamSyntax {
            born: true,
            pair: HashMap::new(),
            stream: stream,
        }
    }
}

impl<'s> Iterator for StreamSyntax<'s>
{
    type Item = HttpPair<'s>;

    fn next(&mut self) -> Option<Self::Item>
    {
        while !self.pair.is_empty() || self.born {
            let h = http::Request::read_header(self.stream);
            let body = http::Request::read_body(self.stream, h.content_length as usize);

            self.pair.entry(h.request_id)
                .or_insert(HttpPair(
                    http::Request::new(self.stream),
                    http::Response::new(self.stream, h.request_id),
                ));

            if h.content_length == 0 && h.type_ == fastcgi::PARAMS {
                self.born = false;
                return self.pair.remove(&h.request_id);
            }

            self.pair
                .get_mut(&h.request_id)
                .expect("HttpPair")
                .request()
                .parse_record(h, body);
        }

        None
    }
}

pub trait Handler: Send + Clone + 'static
{
    fn new() -> Self;

    fn process(&self, &mut HttpPair);
}





