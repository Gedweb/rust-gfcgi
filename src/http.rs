//! HTTP implementation
use fastcgi;
use fastcgi::{Readable, Writable};

use std::io;
use std::collections::HashMap;
use std::net::TcpStream;

extern crate byteorder;
use self::byteorder::{ByteOrder, BigEndian};

#[derive(Debug)]
pub struct Request<'s>
{
    id: u16,
    role: u16,
    flags: u8,
    headers: HashMap<Vec<u8>, Vec<u8>>,
    buf: Vec<u8>,
    stream: &'s TcpStream,
}

impl<'s> Request<'s>
{

    /// Constructor
    pub fn new(id: u16, stream: &TcpStream) -> Request
    {
        Request {
            id: id,
            role: 0,
            flags: 0,
            headers: HashMap::new(),
            buf: Vec::new(),
            stream: stream,
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
}

impl<'s> io::Read for Request<'s>
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize>
    {
        Ok(0)
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
    fn new(data: Vec<u8>) -> Self
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
pub struct Response
{
    id: u16,
    header: HashMap<Vec<u8>, Vec<u8>>,
    body: Vec<u8>,
}

impl Response
{
    /// Constructor
    pub fn new(id: u16) -> Response
    {
        let mut header = HashMap::new();
        header.insert(Vec::from(HTTP_STATUS.as_bytes()),
                      Vec::from("200".as_bytes()));

        Response {
            id: id,
            header: header,
            body: Vec::new(),
        }
    }

    /// Get as raw bytes
    pub fn get_data(&self) -> Vec<u8>
    {
        let mut result: Vec<u8> = Vec::new();

        // http headers
        let mut data: Vec<u8> = Vec::new();

        for (name, value) in &self.header {
            data.extend_from_slice(&name[..]);
            data.push(b':');
            data.extend_from_slice(&value[..]);
            data.extend_from_slice(HTTP_LINE.as_bytes());
        }

        // http headers delimiter
        data.extend_from_slice(HTTP_LINE.as_bytes());

        // http body
        data.extend_from_slice(&self.body);

        for part in data[..].chunks(fastcgi::MAX_LENGTH) {
            result.extend_from_slice(&self.record_header(fastcgi::STDOUT, part.len() as u16));
            result.extend_from_slice(&part);
        }

        // terminate record
        result.extend_from_slice(&self.record_header(fastcgi::STDOUT, 0));
        result.extend_from_slice(&self.end_request());

        result
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

    /// Set some data into response
    pub fn body(&mut self, data: &[u8]) -> &mut Response
    {
        self.body.clear();
        self.body.extend_from_slice(data);

        self
    }

    /// Add some HTTP header
    pub fn header(&mut self, key: &[u8], value: &[u8]) -> &mut Response
    {
        self.header.insert(Vec::from(key), Vec::from(value));

        self
    }

    /// Set custom HTTP status
    pub fn status(&mut self, code: u16) -> &mut Response
    {
        self.header.insert(Vec::from(HTTP_STATUS.as_bytes()),
                           Vec::from(code.to_string().as_bytes())
        );

        self
    }
}
