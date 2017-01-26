//! HTTP implementation
use fastcgi;
use fastcgi::{Readable, Writable};

use std::io;
use std::io::{Read, Write};
use std::collections::HashMap;
use std::net::TcpStream;

extern crate byteorder;
use self::byteorder::{ByteOrder, BigEndian};

#[derive(Debug)]
pub struct Request<'sr>
{
    id: u16,
    role: u16,
    flags: u8,
    headers: HashMap<Vec<u8>, Vec<u8>>,
    buf: Vec<u8>,
    stream: &'sr TcpStream,
    pending: bool,
}

impl<'sr> Request<'sr>
{

    /// Constructor
    pub fn new(stream: &'sr TcpStream, id: u16) -> Request
    {
        Request {
            id: id,
            role: 0,
            flags: 0,
            headers: HashMap::new(),
            buf: Vec::new(),
            stream: stream,
            pending: true,
        }
    }

    /// Add request options
    pub fn add_options(&mut self, data: Vec<u8>)
    {
        let begin_request = fastcgi::BeginRequestBody::read(&data[..]);
        self.role = begin_request.role;
        self.flags = begin_request.flags;
    }

    /// Add param pairs
    pub fn add_param(&mut self, data: Vec<u8>)
    {
        self.headers.extend(ParamFetcher::new(data).parse_param());
    }

    /// FastCGI requestId
    pub fn get_id(&self) -> u16
    {
        self.id
    }

    /// List all headers in bytes
    pub fn headers(&self) -> &HashMap<Vec<u8>, Vec<u8>>
    {
        &self.headers
    }

    /// List all headers in utf8
    pub fn headers_utf8(&self) -> HashMap<String, String>
    {
        self.headers.iter()
            .map(|(k, v)| (
                String::from_utf8_lossy(k).into_owned(),
                String::from_utf8_lossy(v).into_owned(),
            ))
            .collect::<HashMap<_, _>>()
    }

    /// Header by key in bytes
    /// Key are case-sensitive
    pub fn header(&self, key: &[u8]) -> Option<&Vec<u8>>
    {
        self.headers.get(key)
    }

    /// Header by key in utf8
    /// Key are case-sensitive
    pub fn header_utf8(&self, key: &[u8]) -> Option<String>
    {
        self.headers.get(key).map(|v| String::from_utf8_lossy(v).into_owned())
    }

    /// A vector with multiple header in utf8
    /// Key are case-sensitive
    pub fn header_multiple_utf8(&self, key: &[u8]) -> Option<Vec<String>>
    {
        self.header_utf8(key).map(|v| {
                v.split(',')
                .map(|h| h.trim().to_string())
                .collect()
        })
    }

    /// Read FastCGI header
    pub fn fcgi_header(mut stream: &TcpStream) -> fastcgi::Header
    {
        let mut buf: [u8; fastcgi::HEADER_LEN] = [0; fastcgi::HEADER_LEN];
        stream.read(&mut buf).expect("Read fcgi header");
        fastcgi::Header::read(&buf)
    }

    pub fn fcgi_body(stream: &TcpStream, h: &fastcgi::Header) -> Vec<u8>
    {
        let body = match h.content_length {
            0 => Vec::new(),
            _ => Self::stream_read(stream, h.content_length as usize),
        };

        if h.padding_length > 0 {
            Self::stream_read(stream, h.padding_length as usize);
        }

        body
    }

    pub fn stream_read(mut stream: &TcpStream, length: usize) -> Vec<u8>
    {
        let mut body: Vec<u8> = Vec::with_capacity(length);
        unsafe {
            body.set_len(length);
        }

        match stream.read(&mut body) {
            Ok(_len) if _len == length => body,
            Ok(_len) => panic!("{} bytes readed, expected {}", _len, length),
            Err(e)  => panic!("{}", e),
        }
    }

    pub fn fcgi_record(&mut self, h: fastcgi::Header, body: Vec<u8>)
    {
        match h.type_ {
            fastcgi::BEGIN_REQUEST => self.add_options(body),
            fastcgi::PARAMS => self.add_param(body),
            fastcgi::STDIN => self.buf.extend(body),
            _ => panic!("Wrong FastCGI request header"),
        }
    }
}

impl<'sr> io::Read for Request<'sr>
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize>
    {
        while self.buf.len() < buf.len() && self.pending {
            let h = Self::fcgi_header(self.stream);
            if h.content_length == 0 {
                self.pending = false;
                break;
            }
            let body = Self::fcgi_body(self.stream, &h);
            self.buf.extend(body);
        }

        let end = if buf.len() > self.buf.len() {
            self.buf.len()
        } else {
            buf.len()
        };

        // TODO: how avoid it?
        for (k, v) in self.buf.drain(..end).enumerate() {
            buf[k] = v;
        }

        Ok(end)
    }
}


/// Helper for split key-value param pairs
struct ParamFetcher
{
    data: Vec<u8>,
    pos: usize,
}

impl ParamFetcher
{
    /// Constructor
    fn new(data: Vec<u8>) -> ParamFetcher
    {
        ParamFetcher {
            data: data,
            pos: 0,
        }
    }

    /// Parse pairs
    fn parse_param(&mut self) -> HashMap<Vec<u8>, Vec<u8>>
    {
        let mut param: HashMap<Vec<u8>, Vec<u8>> = HashMap::new();

        let data_length: usize = self.data.len();

        while data_length > self.pos {

            let key_length = self.param_length();
            let value_length = self.param_length();

            let key = Vec::from(&self.data[self.pos..self.pos + key_length]);
            self.pos += key_length;

            let value = Vec::from(&self.data[self.pos..self.pos + value_length]);
            self.pos += value_length;

            param.insert(key, value);
        }

        param
    }

    /// Read param length and move interlal cursor
    fn param_length(&mut self) -> usize
    {
        let mut length: usize = self.data[self.pos] as usize;

        if (length >> 7) == 1 {

            self.data[self.pos] = self.data[self.pos] & 0x7F;
            length = BigEndian::read_u32(&self.data[self.pos..(self.pos + 4)]) as usize;

            self.pos += 4;
        } else {
            self.pos += 1;
        }

        length
    }
}



/// HTTP status header
const HTTP_STATUS: &'static str = "Status";
/// HTTP line delimiter
const HTTP_LINE: &'static str = "\r\n";

#[derive(Debug)]
/// HTTP implementation of response
pub struct Response<'sw>
{
    id: u16,
    header: HashMap<Vec<u8>, Vec<u8>>,
    buf: Vec<u8>,
    stream: &'sw TcpStream,
    pending: bool,
}

impl<'sw> Response<'sw>
{
    /// Constructor
    pub fn new(stream: &'sw TcpStream, id: u16) -> Response
    {
        let mut header = HashMap::new();
        header.insert(Vec::from(HTTP_STATUS.as_bytes()),
                      Vec::from("200".as_bytes()));

        Response {
            id: id,
            header: header,
            buf: Vec::new(),
            stream: stream,
            pending: false,
        }
    }

    /// Get as raw bytes
    pub fn http_headers(&self) -> Vec<u8>
    {
        let mut data: Vec<u8> = Vec::new();

        for (name, value) in &self.header {
            data.extend_from_slice(&name[..]);
            data.push(b':');
            data.extend_from_slice(&value[..]);
            data.extend_from_slice(HTTP_LINE.as_bytes());
        }

        // http headers delimiter
        data.extend_from_slice(HTTP_LINE.as_bytes());

        data
    }

    /// End request record
    fn end_request(&self) -> Vec<u8>
    {
        let data = fastcgi::EndRequestBody {
                       app_status: 0,
                       protocol_status: fastcgi::REQUEST_COMPLETE,
                       reserved: [0; 3],
                   }
                   .write();

        let mut result: Vec<u8> = self.record_header(fastcgi::END_REQUEST, data.len() as u16);
        result.extend(data);

        result
    }

    /// Get raw header bytes
    fn record_header(&self, type_: u8, length: u16) -> Vec<u8>
    {
        let header = fastcgi::Header {
            version: fastcgi::VERSION_1,
            type_: type_,
            request_id: self.id,
            content_length: length,
            padding_length: 0,
            reserved: [0; 1],
        };

        header.write()
    }

    /// Add some HTTP header
    pub fn header(&mut self, key: &[u8], value: &[u8])
    {
        self.header.insert(Vec::from(key), Vec::from(value));
    }

    /// Add some HTTP header from utf8
    pub fn header_utf8(&mut self, key: &str, value: &str)
    {
        self.header.insert(key.as_bytes().to_vec(), value.as_bytes().to_vec());
    }

    /// Set custom HTTP status
    pub fn status(&mut self, code: u16)
    {
        self.header.insert(Vec::from(HTTP_STATUS.as_bytes()),
                           Vec::from(code.to_string().as_bytes())
        );
    }

    fn send_header(&mut self)
    {
        if !self.pending {
            for part in self.http_headers().chunks(fastcgi::MAX_LENGTH) {
                let header = self.record_header(fastcgi::STDOUT, part.len() as u16);
                self.stream.write(&header).expect("Send response headers");
                self.stream.write(&part).expect("Send response headers");
            }

            self.pending = true;
        }
    }

    fn send_chunk(&mut self, end: usize)
    {
        let h = self.record_header(fastcgi::STDOUT, end as u16);
        self.stream.write(&h).expect("Send response body");
        self.stream.write(&self.buf.drain(..end).collect::<Vec<_>>()).expect("Send response body");
    }
}

impl<'sw> io::Write for Response<'sw>
{

    fn write(&mut self, buf: &[u8]) -> io::Result<usize>
    {
        self.send_header();
        self.buf.extend_from_slice(buf);
        while self.buf.len() > fastcgi::MAX_LENGTH {
            self.send_chunk(fastcgi::MAX_LENGTH);
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()>
    {
        let mut data: Vec<u8> = Vec::new();

        self.send_header();

        // self a rest
        let end = self.buf.len();
        self.send_chunk(end);

        // terminate record
        data.extend_from_slice(&self.record_header(fastcgi::STDOUT, 0));
        data.extend_from_slice(&self.end_request());

        self.stream.write(&data).expect("Send end");

        Ok(())
    }
}





