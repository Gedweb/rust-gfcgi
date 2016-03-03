use std::net::TcpStream;
use std::io::{Read, Write};

use fcgi::model;

pub struct Stream
{
    _stream: TcpStream,
    request_list: Vec<model::Request>,
}

impl Stream
{
    pub fn new (stream: TcpStream) -> Stream
    {
        Stream {
            _stream: stream,
            request_list: Vec::with_capacity(1024),
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
       'read: loop {

            let mut buf: [u8; model::HEADER_LEN] = [0; model::HEADER_LEN];

            match self._stream.read(&mut buf) {
                Ok(model::HEADER_LEN) => (),
                _ => panic!("fcgi: broken request message"),
            }

            let header = model::Header::read(&buf);
            let request_id = header.request_id as usize;

            let r: &mut model::Request = match self.request_list.get(request_id) {
                Some(..) => &mut self.request_list[request_id],
                None => {
                    self.request_list.insert(request_id, model::Request::new());
                    &mut self.request_list[request_id]
                }
            };

            if header.content_length == 0 {
                match header.type_ {
                    model::STDIN => return Some(r.clone()),
                    model::PARAMS | model::DATA => continue 'read,
                    _ => (),
                }
            }

            let mut body_data: Vec<u8> = Vec::with_capacity(header.content_length as usize);

            match self._stream.read(&mut body_data) {
                Ok(..) => (),
                _  => panic!("fcgi: broken request message"),
            }

            r.add_record(header, body_data);

        };
    }
}






