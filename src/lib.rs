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
                        for id in reader.next() {
                            handler.process(&mut reader);
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
    buf: Vec<u8>,
    last_id: u16,
    request_list: HashMap<u16, model::Request>,
}

impl StreamReader
{
    fn new(stream: TcpStream) -> Self
    {
        StreamReader {
            stream: stream,
            buf: Vec::new(),
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

    fn read_body(&mut self, header: &model::Header) -> Vec<u8>
    {
        let len: usize = header.content_length as usize;
        let mut buf: Vec<u8> = Vec::with_capacity(len);
        unsafe {
            buf.set_len(len);
        }

        match self.stream.read(&mut buf) {
            Ok(readed_len) if readed_len == len => buf,
            Ok(readed_len) => panic!("{} bytes readed, expected {}", readed_len, len),
            Err(e)  => panic!("{}", e),
        }
    }

    pub fn get(&self, item: &[u8]) -> Option<&Vec<u8>>
    {
        self.request_list.get(&self.last_id).unwrap().headers().get(item)
    }

    fn next(&mut self) -> Option<u16>
    {
        while self.last_id == 0 || !self.request_list.is_empty() {
            let h = self.read_header();
            let body = self.read_body(&h);
            let mut r = self.request_list.entry(h.request_id)
                .or_insert(model::Request::new(h.request_id));

            match h.type_ {
                model::BEGIN_REQUEST => r.options(body),
                model::PARAMS => r.param(body),
                model::STDIN | model::DATA => {
                    self.buf.extend(body);
                    self.last_id = h.request_id;
                    return Some(h.request_id.clone());
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
        while buf.len() < self.buf.len() && !self.request_list.get_mut(&self.last_id).unwrap().has_readed() {
            let h = self.read_header();
            if h.content_length == 0 {
                self.request_list.get_mut(&self.last_id).unwrap().mark_readed();
                break;
            }

            let data = self.read_body(&h);
            self.buf.extend(data);
        }

        // TODO: how avoid it?
        let end = if buf.len() > self.buf.len() {
            self.buf.len()
        } else {
            buf.len()
        };

        for (k, v) in self.buf.drain(..end).enumerate() {
            buf[k] = v;
        }

        Ok(end)
    }
}

pub trait Handler: Send + Clone + 'static
{
    fn process(&self, &mut StreamReader);
}











