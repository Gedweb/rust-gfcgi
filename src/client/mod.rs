pub mod model;

use std::net::TcpStream;
use std::io::{Read, Write};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Stream
{
    _stream: TcpStream,
    request_list: HashMap<u16, model::Request>,
    readable: bool,
}

impl Stream
{
    pub fn new (stream: TcpStream) -> Stream
    {
        Stream {
            _stream: stream,
            request_list: HashMap::new(),
            readable: true,
        }
    }

    pub fn write(&mut self, response: &model::Response)
    {
        match self._stream.write(&response.get_data()) {
            Ok(_) => (),
            _ => panic!("fcgi: failed sending response"),
        }
    }
}

impl Iterator for Stream
{
    type Item = model::Request;

    fn next(&mut self) -> Option<model::Request>
    {
        let mut result: Option<model::Request> = None;

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
                    self.request_list.insert(request_id.clone(), model::Request::new(request_id.clone()));
                },
            };

            if header.content_length == 0 {
                match header.type_ {
                    model::STDIN => {
                        result = self.request_list.remove(&request_id);
                        break;
                    },
                    model::PARAMS | model::DATA => continue 'read,
                    _ => (),
                };
            }

            let len: usize = header.content_length as usize;
            let mut body_data: Vec<u8> = Vec::with_capacity(len);
            unsafe {
                body_data.set_len(len);
            }

            match self._stream.read(&mut body_data) {
                Ok(readed_len) if readed_len == len =>
                    self.request_list.get_mut(&request_id).unwrap().add_record(&header, body_data),
                _  => panic!("Wrong body length"),
            }
        };

        if self.request_list.is_empty() {
            self.readable = false;
        }

        result
    }
}
