use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;

use fcgi::entity;
use fcgi::serialize::Serialize;

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
}

impl Stream
{
    fn new (stream: TcpStream) -> Stream
    {
        Stream {
            _stream: stream,
        }
    }
    
    fn handle(&mut self)
    {
        let mut buf: [u8; entity::HEADER_LEN] = [0; entity::HEADER_LEN]; 
        
        match self._stream.read(&mut buf) {
            Ok(entity::HEADER_LEN) => (),
            _ => panic!("Broken message"),
        }
        
        let mut header = entity::Header::new();
        header.read(&buf);
        
        println!("{:?}", header);
    }
}