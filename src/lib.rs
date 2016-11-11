#![allow(dead_code)]
//! This crate provieds FastCGI client with supporting multithreaded socket listener and HTTP-instances multiplexed onto a single connection.

// object
mod fastcgi;
mod http;

pub use http::{Request, Response};

use fastcgi::{Readable};

// Data struct
use std::collections::HashMap;
use std::iter::Iterator;

// io
use std::io;
use std::io::Read;
use std::net::{TcpListener, TcpStream, ToSocketAddrs};

// Thread
use std::thread;

pub struct Client<T>
{
    listener: TcpListener,
    handler: T,
}

impl<T: Handler> Client<T>
{
    pub fn new<A: ToSocketAddrs>(addr: A, handler: T) -> Self
    {
        Client {
            listener: TcpListener::bind(addr).expect("Bind address"),
            handler: handler,
        }
    }

    pub fn run(&self)
    {
        let listener = self.listener.try_clone().expect("Clone listener");
        let handler  = self.handler.clone();

        thread::spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        let mut reader = StreamReader::new(stream);
                        for mut request in reader.next() {

                            // call handler
                            let response = handler.process(&mut request, &mut reader);

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

pub struct StreamReader
{
    stream: TcpStream,
    buf: Vec<u8>,
    last_id: u16,
    request: HashMap<u16, http::Request>,
}

impl StreamReader
{
    fn new(stream: TcpStream) -> Self
    {
        StreamReader {
            stream: stream,
            buf: Vec::new(),
            last_id: 0,
            request: HashMap::new(),
        }
    }

    fn read_header(&mut self) -> fastcgi::Header
    {
        let mut buf: [u8; fastcgi::HEADER_LEN] = [0; fastcgi::HEADER_LEN];
        self.stream.read(&mut buf).expect("Read fcgi header");
        fastcgi::Header::read(&buf)
    }

    fn read_body(&mut self, header: &fastcgi::Header) -> Vec<u8>
    {
        let len: usize = header.content_length as usize;
        let mut buf: Vec<u8> = Vec::with_capacity(len);
        unsafe {
            buf.set_len(len);
        }

        match self.stream.read(&mut buf) {
            Ok(_len) if _len == len => buf,
            Ok(_len) => panic!("{} bytes readed, expected {}", _len, len),
            Err(e)  => panic!("{}", e),
        }
    }
//}
//
//impl Iterator for StreamReader
//{
//    type Item = http::Request;

    fn next(&mut self) -> Option<http::Request>
    {
        while self.last_id == 0 || !self.request.is_empty() {
            let h = self.read_header();
            let body = self.read_body(&h);
            self.request.entry(h.request_id).or_insert(http::Request::new(h.request_id));

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
                    self.buf.extend(body);
                    self.last_id = h.request_id;
                    return self.request.remove(&h.request_id);
                }
                _ => panic!("Undeclarated fastcgi header"),
            }
        }

        None
    }
}

impl io::Read for StreamReader
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize>
    {
        while buf.len() < self.buf.len() && !self.request.get(&self.last_id).expect("HTTP Request body").has_readed() {
            let h = self.read_header();
            if h.content_length == 0 {
                self.request.get_mut(&self.last_id).expect("HTTP Request readed").mark_readed();
                break;
            }

            let data = self.read_body(&h);
            self.buf.extend(data);
        }

        let end = if buf.len() > self.buf.len() {
            self.buf.len()
        } else {
            buf.len()
        };

        // TODO: how avoid it?
        for (k, v) in self.buf.drain(..end).enumerate() {
            buf[k] = v;
        }

        Ok(end)
    }
}

pub trait Handler: Send + Clone + 'static
{
    fn process(&self, &mut Request, &mut StreamReader) -> Option<Response>;
}





