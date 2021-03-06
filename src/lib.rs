#![allow(dead_code)]
//! This crate provides FastCGI client with supporting multithreaded socket listener and HTTP-instances multiplexed into a single connection.
// object
mod fastcgi;
mod http;

pub use http::{Request, Response};

// Data struct
use std::collections::HashMap;
use std::iter::Iterator;

// net / io
use std::net::{TcpListener, TcpStream, ToSocketAddrs, Incoming};
use std::io::Write;

// Thread
#[cfg(feature="spawn")]
use std::thread;

pub struct Client
{
    listener: TcpListener,
}

/// TcpListener wrapper
impl Client
{
    pub fn new<A: ToSocketAddrs>(addr: A) -> Client
    {
        Client {
            listener: TcpListener::bind(addr).expect("Bind address"),
        }
    }

    /// Run thread
    /// Accept `Handler` as callback
    #[cfg(feature="spawn")]
    pub fn run<T: Handler + Send + Clone + 'static>(&self, handler: T) -> std::thread::JoinHandle<()>
    {
        let listener = self.listener.try_clone().expect("Clone listener");
        let handler = handler.clone();

        thread::spawn(move || {
            Self::listen(listener.incoming(), handler);
        })
    }

    /// Accept `Handler` as callback
    #[cfg(not(feature="spawn"))]
    pub fn run<T: Handler>(&self, handler: T)
    {
        Self::listen(self.listener.incoming(), handler);
    }

    fn listen<T: Handler>(incoming: Incoming, handler: T)
    {
        for stream in incoming {
            match stream {
                Ok(stream) => {
                    let reader = StreamSyntax::new(&stream);
                    for pair in reader {

                        // call handler
                        let (mut request, mut response) = pair;
                        handler.process(&mut request, &mut response);

                        response.flush().unwrap();
                    }
                }
                Err(e) => panic!("{}", e),
            }
        }
    }
}

/// HTTP request / response pairs
pub type HttpPair<'s> = (Request<'s>, Response<'s>);

/// FasctCGI request parser
pub struct StreamSyntax<'s>
{
    born: bool,
    pair: HashMap<u16, HttpPair<'s>>,
    stream: &'s TcpStream,
}

impl<'s> StreamSyntax<'s>
{
    fn new(stream: &'s TcpStream) -> StreamSyntax
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
            let h = Request::fcgi_header(self.stream);
            let body = Request::fcgi_body(self.stream, &h);

            self.pair.entry(h.request_id)
                .or_insert((
                    Request::new(self.stream, h.request_id),
                    Response::new(self.stream, h.request_id),
                ));

            match h.type_ {
                fastcgi::ABORT_REQUEST => {
                    self.pair.remove(&h.request_id).unwrap()
                        .1.flush()
                        .expect("Send end request on abort")
                }
                fastcgi::PARAMS if h.content_length == 0 => {
                    self.born = false;
                    return self.pair.remove(&h.request_id);
                }
                _ => {
                    self.pair.get_mut(&h.request_id).unwrap()
                        .0.fcgi_record(h, body)
                }
            }
        }

        None
    }
}

/// Callback trait
pub trait Handler
{
    /// Run HTTP-request handling
    fn process(&self, request: &mut Request, response: &mut Response);
}





