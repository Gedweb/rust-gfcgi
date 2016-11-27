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

/// TcpListener wrapper
impl Client
{
    pub fn new<A: ToSocketAddrs>(addr: A) -> Self
    {
        Client {
            listener: TcpListener::bind(addr).expect("Bind address"),
        }
    }

    /// Run thread
    /// Accept `Handler` as callback
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

/// HTTP request / response pairs
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

/// FasctCGI request parser
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

/// Iterator implementation
impl<'s> Iterator for StreamSyntax<'s>
{
    type Item = HttpPair<'s>;

    /// Yield HTTP request / response
    fn next(&mut self) -> Option<Self::Item>
    {
        while !self.pair.is_empty() || self.born {
            let h = http::Request::fcgi_header(self.stream);
            let body = http::Request::fcgi_body(self.stream, h.content_length as usize);

            self.pair.entry(h.request_id)
                .or_insert(HttpPair(
                    http::Request::new(self.stream, h.request_id),
                    http::Response::new(self.stream, h.request_id),
                ));

            match h.type_ {
                fastcgi::ABORT_REQUEST => {
                    self.pair.remove(&h.request_id).unwrap()
                        .response()
                        .flush()
                        .expect("Send end request on abort")
                }
                fastcgi::PARAMS if h.content_length == 0 => {
                    self.born = false;
                    return self.pair.remove(&h.request_id);
                }
                _ => {
                    self.pair.get_mut(&h.request_id).unwrap()
                        .request().fcgi_record(h, body)
                }
            }
        }

        None
    }
}

/// Callback trait
pub trait Handler: Send + Clone + 'static
{
    /// Return new instance
    fn new() -> Self;

    /// Run HTTP-request handling
    fn process(&self, &mut HttpPair);
}





