#![allow(dead_code)]

pub mod model;

use std::collections::HashMap;

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use std::thread;
use std::sync::mpsc;

use model::{Readable, Writable};

pub struct Client
{
    listener: TcpListener,
    list: Vec<model::Request>,
    request_tx: mpsc::Sender<model::Request>,
    request_rx: mpsc::Receiver<model::Request>,
}

impl Client
{
    pub fn new(listener: TcpListener) -> Client
    {
        let (tx, rx) = mpsc::channel();
        Client {
            listener: listener,
            list: Vec::new(),
            request_tx: tx,
            request_rx: rx,
        }
    }

    pub fn run(&self)
    {
        let request_tx = self.request_tx.clone();
        let listener = self.listener.try_clone().unwrap();

        thread::spawn(move || {

            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        let mut fcgi_stream = Stream::new(stream, request_tx.clone());
                        fcgi_stream.read();
                        fcgi_stream.write();

                    },
                    Err(msg) => panic!("{}", msg),
                }
            }
        });
    }
}

impl Iterator for Client
{
    type Item = model::Request;

    fn next(&mut self) -> Option<model::Request>
    {
        Some(self.request_rx.recv().unwrap())
    }

}

#[derive(Debug)]
pub struct Stream
{
    _stream: TcpStream,
    request_count: u16,
    client_tx: mpsc::Sender<model::Request>,
    tx: mpsc::Sender<model::Response>,
    rx: mpsc::Receiver<model::Response>,
    readable: bool,
}

impl Stream
{
    fn new (stream: TcpStream, client_tx: mpsc::Sender<model::Request>) -> Stream
    {
        let (tx, rx) = mpsc::channel();
        Stream {
            _stream: stream,
            request_count: 0,
            client_tx: client_tx,
            tx: tx,
            rx: rx,
            readable: true,
        }
    }

    fn write(&mut self)
    {
        while self.request_count != 0 {
            let response = self.rx.recv().unwrap();
            self._stream.write(&response.get_data()).unwrap();
            self.request_count -= 1;
        }
    }

    fn read(&mut self)
    {
        let mut request_list: HashMap<u16, model::Request> = HashMap::new();

        'read: while self.readable {

            let mut buf: [u8; model::HEADER_LEN] = [0; model::HEADER_LEN];

            match self._stream.read(&mut buf) {
                Ok(model::HEADER_LEN) => (),
                _ => panic!("FCGI header mismatch"),
            };

            let header = model::Header::read(&buf);
            let request_id = header.request_id.clone();

            match request_list.get(&request_id) {
                Some(..) => (),
                None => {
                    request_list.insert(request_id.clone(), model::Request::new(request_id.clone(), self.tx.clone()));
                    self.request_count += 1;
                },
            };

            let len: usize = header.content_length as usize;
            let mut body_data: Vec<u8> = Vec::with_capacity(len);

            if header.content_length == 0 {
                match header.type_ {
                    model::STDIN => {
                        self.client_tx.send(request_list.remove(&request_id).unwrap()).unwrap();
                    },
                    model::PARAMS | model::DATA => continue 'read,
                    _ => (),
                };
            } else {

                unsafe {
                    body_data.set_len(len);
                }

                match self._stream.read(&mut body_data) {
                    Ok(readed_len) if readed_len == len =>
                        request_list.get_mut(&request_id).unwrap().add_record(&header, body_data),
                    _  => panic!("Wrong body length"),
                }
            }

            if request_list.is_empty() {
                self.readable = false;
            }
        }
    }
}
