pub mod model;

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use std::collections::HashMap;

use std::thread;
use std::sync::mpsc;
// use std::sync::Arc;

type Request = model::Request<Stream>;

pub struct Client
{
    listener: TcpListener,
    list: Vec<Request>,
    request_tx: mpsc::Sender<Request>,
    request_rx: mpsc::Receiver<Request>,
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

    pub fn init(&self)
    {
        let request_tx = self.request_tx.clone();
        let listener = self.listener.try_clone().unwrap();

        thread::spawn(move || {

            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        let mut fcgi_stream = Stream::new(stream, request_tx.clone());
                        fcgi_stream.read();

                    },
                    Err(msg) => panic!("{}", msg),
                }
            }
        });
    }
}

impl Iterator for Client
{
    type Item = model::Request<Stream>;

    fn next(&mut self) -> Option<Request>
    {
        Some(self.request_rx.recv().unwrap())
    }

}

#[derive(Debug)]
pub struct Stream
{
    _stream: TcpStream,
    request_list: HashMap<u16, Request>,
    parent_tx: mpsc::Sender<Request>,
    tx: mpsc::Sender<Stream>,
    rx: mpsc::Receiver<Stream>,
    readable: bool,
}

impl Stream
{
    fn new (stream: TcpStream, parent_tx: mpsc::Sender<Request>) -> Stream
    {
        let (tx, rx) = mpsc::channel();
        Stream {
            _stream: stream,
            request_list: HashMap::new(),
            parent_tx: parent_tx,
            tx: tx,
            rx: rx,
            readable: true,
        }
    }

    pub fn write(&mut self, response: &model::Response<&Request>)
    {
        match self._stream.write(&response.get_data()) {
            Ok(_) => (),
            _ => panic!("fcgi: failed sending response"),
        }
    }

    pub fn read(&mut self)
    {
        'read: while self.readable {

            let mut buf: [u8; model::HEADER_LEN] = [0; model::HEADER_LEN];

            match self._stream.read(&mut buf) {
                Ok(model::HEADER_LEN) => (),
                _ => panic!("FCGI header mismatch"),
            };

            let header = model::Header::read(&buf);
            let request_id = header.request_id.clone();

            match self.request_list.get(&request_id) {
                Some(..) => (),
                None => {
                    self.request_list.insert(request_id.clone(), model::Request::new(request_id.clone(), self.tx.clone()));
                },
            };

            let len: usize = header.content_length as usize;
            let mut body_data: Vec<u8> = Vec::with_capacity(len);

            if header.content_length == 0 {
                match header.type_ {
                    model::STDIN => {
                        self.parent_tx.send(self.request_list.remove(&request_id).unwrap()).unwrap();
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
                        self.request_list.get_mut(&request_id).unwrap().add_record(&header, body_data),
                    _  => panic!("Wrong body length"),
                }
            }

            if self.request_list.is_empty() {
                self.readable = false;
            }
        }
    }
}
