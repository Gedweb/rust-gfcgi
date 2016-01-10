use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;

use fcgi::model;
use std::collections::HashMap;

pub struct Listener
{
    _listener: TcpListener,
}

impl Listener
{
    
    pub fn new(host_port: &str) -> Listener
    {
    
        let listener = match TcpListener::bind(host_port) {
            Ok(result) => result,
            Err(_) => panic!("Can't bind {}", host_port),
        };
        
        Listener {
            _listener: listener,
        }
    }

    pub fn run(&self) -> &Listener
    {
      
        for stream in self._listener.incoming() {
            
            let child = match stream {
                Ok(stream) => thread::spawn(move || {
                    Stream::new(stream).handle();
                }),
                Err(error) => panic!("Connection error {}", error),
            };
            
            child.thread();
        }
        
        self
    }
}

pub struct Stream
{
    _stream: TcpStream,
    param_string: String,
}

impl Stream
{
    fn new (stream: TcpStream) -> Stream
    {
        Stream {
            _stream: stream,
            param_string: String::new(),
        }
    }
    
    fn handle(&mut self)
    {
        let request_list: HashMap<u16, model::Request> = self.read();
        
        for (request_id, request) in request_list {
            self.write(request_id);
        }  
    }
    
    fn write(&mut self, request_id: u16)
    {
        let response = model::Response::new(request_id, b"Status: 401\r\nContent-Type: text/plain\r\n\r\nIt's work!".to_vec());
        
        match self._stream.write(&response.get_data()) {
            Ok(_) => (),
            _ => panic!("Broken message"),
        }
    }
    
    fn read(&mut self) -> HashMap<u16, model::Request>
    {
        let mut request_list: HashMap<u16, model::Request> = HashMap::new();
        
        'read: loop {
            
            let mut buf: [u8; model::HEADER_LEN] = [0; model::HEADER_LEN];

            match self._stream.read(&mut buf) {
                Ok(model::HEADER_LEN) => (),
                _ => panic!("Broken message"),
            }
            
            let header = model::Header::read(&buf);
            
            let r: &mut model::Request = match request_list.contains_key(&header.request_id) {    
                true => request_list.get_mut(&header.request_id).unwrap(),
                false => {
                    request_list.insert(header.request_id, model::Request::new());
                    request_list.get_mut(&header.request_id).unwrap()
                }
            };
            
            // @todo break on all stream types
            if header.content_length == 0 {
                match header.type_ {
                    model::STDIN => break 'read,
                    model::PARAMS | model::DATA => continue 'read,
                    _ => (),
                }
                break 'read;
            }
            
            let body_data = self.read_byte(header.content_length as usize);
			r.add_body(header, body_data);
        
        }
        
//        println!("{:?}", request_list);
        
        request_list
    }
    
    fn read_byte(&mut self, len: usize) -> Vec<u8>
    {
        let mut body_data: Vec<u8> = Vec::with_capacity(len);
    
        match (&self._stream).take(len as u64).read_to_end(&mut body_data) {
            Ok(readed_len) if readed_len == len => body_data,
            _  => panic!("Wrong body length"),
        }
    }
}