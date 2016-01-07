use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;

use fcgi::model;

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
        let mut request = model::Request::new();
        
        'read: loop {
            
            let r = &mut request;
            let mut buf: [u8; model::HEADER_LEN] = [0; model::HEADER_LEN];
			
            match self._stream.read(&mut buf) {
                Ok(model::HEADER_LEN) => (),
                _ => panic!("Broken message"),
            }
            
            let header = model::Header::read(&buf);
            
//            println!("{:?}", header);
            
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

			match header.type_ {
			    model::BEGIN_REQUEST => {
			        
			    },
			    model::PARAMS => r.add_param(body_data),
			    model::STDIN => r.add_body(body_data),
			    model::DATA => r.add_body(body_data),
				_ => (), // panic!("Undeclarated fastcgi header"),
			};
        }
        
        println!("{:?}", request);
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