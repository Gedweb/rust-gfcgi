#![allow(dead_code)]
//! This crate provieds FastCGI client with supporting multithreaded socket listener and HTTP-instances multiplexed onto a single connection.

// object
mod fastcgi;
mod http;

pub use http::{Request, Response};

use fastcgi::{Readable};

// Data struct
use std::collections::HashMap;

// io
use std::io::Read;
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
                        let mut reader = StreamReader::new();
                        for mut request in reader.next(&stream) {

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

pub struct StreamReader<'r>
{
    status: ParseStatus,
    request: HashMap<u16, http::Request<'r>>,
}

impl<'r> StreamReader<'r>
{
    fn new() -> Self
    {
        StreamReader {
            status: ParseStatus::Begin,
            request: HashMap::new(),
        }
    }

    fn next(&mut self, mut stream: &'r TcpStream) -> Option<http::Request>
    {
        while !self.request.is_empty() || self.status == ParseStatus::Begin {

            // read fcgi-header
            let mut buf: [u8; fastcgi::HEADER_LEN] = [0; fastcgi::HEADER_LEN];
            stream.read(&mut buf).expect("Read fcgi header");
            let h = fastcgi::Header::read(&buf);

            // read fcgi-record
            let len: usize = h.content_length as usize;
            let mut body: Vec<u8> = Vec::with_capacity(len);
            unsafe {
                body.set_len(len);
            }
            match stream.read(&mut body) {
                Ok(_len) if _len == len => (),
                Ok(_len) => panic!("{} bytes readed, expected {}", _len, len),
                Err(e)  => panic!("{}", e),
            }

            // parse fcgi-record
            self.request.entry(h.request_id).or_insert(http::Request::new(h.request_id, stream));

            match h.type_ {
                fastcgi::BEGIN_REQUEST => self
                    .request.get_mut(&h.request_id)
                    .expect("HTTP Request begin")
                    .add_options(body),
                fastcgi::PARAMS => self
                    .request.get_mut(&h.request_id)
                    .expect("HTTP Request param")
                    .add_param(body),
                fastcgi::STDIN | fastcgi::DATA => {
//                    self.buf.extend(body);
                    return self.request.remove(&h.request_id);
                }
                _ => panic!("Undeclarated fastcgi header"),
            }
        }

        self.status = ParseStatus::Progress;

        None
    }
}

pub trait Handler: Send + Clone + 'static
{
    fn new() -> Self;

    fn process(&self, &mut Request) -> Option<Response>;
//
//    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize>
//    {
//        while buf.len() < self.buf.len() && !self.request.get(&self.last_id).expect("HTTP Request body").has_readed() {
//            let h = self.read_header();
//            if h.content_length == 0 {
//                self.request.get_mut(&self.last_id).expect("HTTP Request readed").mark_readed();
//                break;
//            }
//
//            let data = self.read_body(&h);
//            self.buf.extend(data);
//        }
//
//        let end = if buf.len() > self.buf.len() {
//            self.buf.len()
//        } else {
//            buf.len()
//        };
//
//        // TODO: how avoid it?
//        for (k, v) in self.buf.drain(..end).enumerate() {
//            buf[k] = v;
//        }
//
//        Ok(end)
//    }
}





