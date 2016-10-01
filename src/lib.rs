#![allow(dead_code)]
//! This crate provieds FastCGI client with supporting multithreaded socket listener and HTTP-instances multiplexed onto a single connection.

// FastCGI entity
mod model;
use model::{Readable};

pub use model::{
    Request,
    Response,
};

// Data struct
use std::collections::HashMap;

// Stream
use std::io::{Read};
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
            listener: TcpListener::bind(addr).unwrap(),
            handler: handler,
        }
    }

    pub fn run(&self)
    {
        let listener = self.listener.try_clone().unwrap();
        let handler  = self.handler.clone();

        thread::spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        for fcgi_request in StreamReader::new(stream).read() {
                            handler.process(fcgi_request);
                        }
                    }
                    Err(msg) => panic!("{}", msg),
                }
            }
        });

        thread::park();
    }
}

#[derive(Debug)]
struct StreamReader
{
    _stream: TcpStream,
    request_list: HashMap<u16, model::Request>
}

impl StreamReader
{
    fn new(stream: TcpStream) -> Self
    {
        StreamReader {
            _stream: stream,
            request_list: HashMap::new(),
        }
    }

    fn read_header(&mut self) -> model::Header
    {
        let mut buf: [u8; model::HEADER_LEN] = [0; model::HEADER_LEN];
        self._stream.read(&mut buf).unwrap();
        model::Header::read(&buf)
    }

    fn read_body(&mut self, header: &model::Header)
    {
        self.request_list.entry(header.request_id).or_insert(model::Request::new(header.request_id));

        let mut r = self.request_list.get_mut(&header.request_id).unwrap();

        let len: usize = header.content_length as usize;
        let mut buf: Vec<u8> = Vec::with_capacity(len);
        unsafe {
            buf.set_len(len);
        }

        match self._stream.read(&mut buf) {
            Ok(readed_len) if readed_len == len =>
                r.add_record(&header, buf),
            Ok(readed_len) => panic!("{} bytes readed, expected {}", readed_len, len),
            Err(e)  => panic!("{}", e),
        }
    }


    fn read(&mut self) -> Option<model::Request>
    {
        loop {
            let h = self.read_header();
            if h.type_ == model::STDIN {

                return self.request_list.remove(&h.request_id)
            }

            self.read_body(&h);
        }
    }
}

pub trait Handler: Send + Clone + 'static
{
    fn process(&self, model::Request);
}















