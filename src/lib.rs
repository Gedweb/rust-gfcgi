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
use std::ops::Index;
use std::collections::HashMap;

// Stream
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
                        let mut reader = StreamReader::new(stream);
                        for id in reader.wait() {
                            handler.process(&reader);
                        }
                    }
                    Err(msg) => panic!("{}", msg),
                }
            }
        });

        thread::park();
    }
}

pub struct StreamReader
{
    stream: TcpStream,
    buf: [u8; model::MAX_LENGTH],
    last_id: u16,
    request_list: HashMap<u16, model::Request>,
}

impl StreamReader
{
    fn new(stream: TcpStream) -> Self
    {
        StreamReader {
            stream: stream,
            buf: [0; model::MAX_LENGTH],
            last_id: 0,
            request_list: HashMap::new(),
        }
    }

    fn read_header(&mut self) -> model::Header
    {
        let mut buf: [u8; model::HEADER_LEN] = [0; model::HEADER_LEN];
        self.stream.read(&mut buf).unwrap();
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

        match self.stream.read(&mut buf) {
            Ok(readed_len) if readed_len == len =>
                r.add_record(&header, buf),
            Ok(readed_len) => panic!("{} bytes readed, expected {}", readed_len, len),
            Err(e)  => panic!("{}", e),
        }
    }


    fn wait(&mut self) -> Option<u16>
    {
        loop {
            let h = self.read_header();
            if h.type_ == model::STDIN {
                self.last_id = h.request_id;
                return Some(h.request_id.clone())
            }

            self.read_body(&h);
        }

        None
    }

    pub fn get(&self, item: &[u8]) -> Option<&Vec<u8>>
    {
        self.request_list.get(&self.last_id).unwrap().headers().get(item)
    }
}

impl io::Read for StreamReader
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize>
    {
        Ok(0)
    }
}

pub trait Handler: Send + Clone + 'static
{
    fn process(&self, &StreamReader);
}











