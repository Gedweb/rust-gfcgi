use std::net::TcpStream;
use std::io::{Read, Write};

use fcgi::model;
use std::collections::HashMap;

pub struct Stream
{
    _stream: TcpStream,
    param_string: String,
}

impl Stream
{
    pub fn new (stream: TcpStream) -> Stream
    {
        Stream {
            _stream: stream,
            param_string: String::new(),
        }
    }
    
    pub fn write(&mut self, request_id: u16)
    {
        let response = model::Response::new(request_id);
        
        match self._stream.write(&response.get_data()) {
            Ok(_) => (),
            _ => panic!("fcgi: failed sending response"),
        }
    }
    
    pub fn read(&mut self) -> HashMap<u16, model::Request>
    {
        let mut request_list: HashMap<u16, model::Request> = HashMap::new();
        
        'read: loop {
            
            let mut buf: [u8; model::HEADER_LEN] = [0; model::HEADER_LEN];

            match self._stream.read(&mut buf) {
                Ok(model::HEADER_LEN) => (),
                _ => panic!("fcgi: broken request message"),
            }
            
            let header = model::Header::read(&buf);
            
            let r: &mut model::Request = match request_list.contains_key(&header.request_id) {
                true => request_list.get_mut(&header.request_id).unwrap(),
                false => {
                    request_list.insert(header.request_id, model::Request::new());
                    request_list.get_mut(&header.request_id).unwrap()
                }
            };
            
            if header.content_length == 0 {
                match header.type_ {
                    model::STDIN => break 'read,
//                    model::PARAMS | model::DATA => continue 'read,
                    _ => (),
                }
            }
            
            let body_data = self.read_byte(header.content_length as usize);
            r.add_body(header, body_data);
        
        }
        
        request_list
    }
    
    fn read_byte(&mut self, len: usize) -> Vec<u8>
    {
        let mut body_data: Vec<u8> = Vec::with_capacity(len);
    
        match (&self._stream).take(len as u64).read_to_end(&mut body_data) {
            Ok(readed_len) if readed_len == len => body_data,
            _  => panic!("fcgi: broken request message"),
        }
    }
}