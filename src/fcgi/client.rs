use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;

use fcgi::entity;

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
                    Stream::new(stream);
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
}