/// Contain constants and models for fcgi data records.

// ----------------- entity -----------------
use std::collections::HashMap;

/// Listening socket file number
pub const LISTENSOCK_FILENO: u8 = 0;

/// FCGI record header
#[derive(Debug)]
pub struct Header
{
    pub version: u8,
    pub type_: u8,
    pub request_id: u16,
    pub content_length: u16,
    pub padding_length: u8,
    reserved: [u8; 1],
}

/// Maximum length per record
pub const MAX_LENGTH: usize = 0xffff;

/// Number of bytes in a Header.
///
/// Future versions of the protocol will not reduce this number.
pub const HEADER_LEN: usize = 8;

/// version component of Header
pub const VERSION_1: u8 = 1;

/// type component of Header
/// # Request
/// The Web server sends a FCGI_BEGIN_REQUEST record to start a request
pub const BEGIN_REQUEST: u8 = 1;

/// type component of Header
/// # Request
/// A Web server aborts a FastCGI request when an HTTP client closes its transport connection while the FastCGI request is running on behalf of that client
pub const ABORT_REQUEST: u8 = 2;

/// type component of Header
/// # Response
/// The application sends a FCGI_END_REQUEST record to terminate a request
pub const END_REQUEST: u8 = 3;

/// type component of Header
/// # Request
/// Receive name-value pairs from the Web server to the application
pub const PARAMS: u8 = 4;

/// type component of Header
/// # Request
/// Byte Stream
pub const STDIN: u8 = 5;

/// type component of Header
/// # Response
/// Byte Stream
pub const STDOUT: u8 = 6;

/// type component of Header
/// # Response
/// Byte Stream
pub const STDERR: u8 = 7;

/// type component of Header
/// # Request
/// Byte Stream
pub const DATA: u8 = 8;

/// type component of Header
/// # Request
/// The Web server can query specific variables within the application
/// The application receives.
pub const GET_VALUES: u8 = 9;

/// type component of Header
/// # Response
/// The Web server can query specific variables within the application.
/// The application responds.
pub const GET_VALUES_RESULT: u8 = 10;

/// type component of Header
///
/// Unrecognized management record
pub const UNKNOWN_TYPE: u8 = 11;

/// Default request id component of Header
pub const NULL_REQUEST_ID: u16 = 0;

/// Begin record
struct BeginRequestBody
{
    role: u16,
    flags: u8,
    reserved: [u8; 5],
}

#[cfg(feature = "struct_record")]
struct BeginRequestRecord
{
    header: Header,
    body: BeginRequestBody,
}

/// Mask for flags component of BeginRequestBody
pub const KEEP_CONN: u8 = 1;

/// FastCGI role
/// emulated CGI/1.1 program
pub const RESPONDER: u8 = 1;

/// FastCGI role
/// authorized/unauthorized decision
pub const AUTHORIZER: u8 = 2;

/// FastCGI role
/// extra stream of data from a file
pub const FILTER: u8 = 3;

/// End record
struct EndRequestBody
{
    app_status: u32,
    protocol_status: u8,
    reserved: [u8; 3],
}

#[cfg(feature = "struct_record")]
struct EndRequestRecord
{
    header: Header,
    body: EndRequestBody,
}

/// protocol_status component of EndRequestBody
///
/// Normal end of request
pub const REQUEST_COMPLETE: u8 = 0;

/// protocol_status component of EndRequestBody
///
/// Application is designed to process one request at a time per connection
pub const CANT_MPX_CONN: u8 = 1;

/// protocol_status component of EndRequestBody
///
/// The application runs out of some resource, e.g. database connections
pub const OVERLOADED: u8 = 2;

/// protocol_status component of EndRequestBody
///
/// Web server has specified a role that is unknown to the application
pub const UNKNOWN_ROLE: u8 = 3;

/// Names for GET_VALUES / GET_VALUES_RESULT records.
///
/// The maximum number of concurrent transport connections this application will accept, e.g. "1" or "10".
pub const MAX_CONNS: &'static str = "MAX_CONNS";

/// Names for GET_VALUES / GET_VALUES_RESULT records.
///
/// The maximum number of concurrent requests this application will accept, e.g. "1" or "50".
pub const MAX_REQS: &'static str = "MAX_REQS";

/// Names for GET_VALUES / GET_VALUES_RESULT records.
///
/// If this application does not multiplex connections (i.e. handle concurrent requests over each connection), "1" otherwise.
pub const MPXS_CONNS: &'static str = "MPXS_CONNS";


struct UnknownTypeBody
{
    type_: u8,
    reserved: [u8; 7],
}

#[cfg(feature = "struct_record")]
struct UnknownTypeRecord
{
    pub header: Header,
    pub body: UnknownTypeBody,
}

// ----------------- repository -----------------

extern crate byteorder;

use self::byteorder::{ByteOrder, BigEndian};

pub trait Readable {
    /// Must to decode bytes to object
    fn read(data: &[u8]) -> Self;
}

pub trait Writable {
    /// Must to encode object to bytes
    fn write(&self) -> Vec<u8>;
}

// ----------------- implementation -----------------

impl Readable for Header
{
    fn read(data: &[u8]) -> Header
    {
        Header {
            version: data[0],
            type_: data[1],
            request_id: BigEndian::read_u16(&data[2..4]),
            content_length: BigEndian::read_u16(&data[4..6]),
            padding_length: data[6],
            reserved: [0; 1],
        }
    }
}

impl Writable for Header
{
    fn write(&self) -> Vec<u8>
    {
        let mut data: Vec<u8> = Vec::with_capacity(self::HEADER_LEN);

        data.push(self.version);
        data.push(self.type_);

        let mut buf: [u8; 2] = [0; 2];
        BigEndian::write_u16(&mut buf, self.request_id);
        data.extend_from_slice(&buf);

        let mut buf: [u8; 2] = [0; 2];
        BigEndian::write_u16(&mut buf, self.content_length);
        data.extend_from_slice(&buf);

        data.push(self.padding_length);
        data.extend_from_slice(&self.reserved);

        data
    }
}


impl Readable for BeginRequestBody
{
    fn read(data: &[u8]) -> BeginRequestBody
    {
        BeginRequestBody {
            role: BigEndian::read_u16(&data[0..2]),
            flags: data[2],
            reserved: [0; 5],
        }
    }
}

impl Writable for BeginRequestBody
{
    fn write(&self) -> Vec<u8>
    {
        let mut data: Vec<u8> = Vec::with_capacity(8);

        let mut buf: [u8; 2] = [0; 2];
        BigEndian::write_u16(&mut buf, self.role);
        data.extend_from_slice(&buf);

        data.push(self.flags);
        data.extend_from_slice(&self.reserved);

        data
    }
}


impl Readable for EndRequestBody
{
    fn read(data: &[u8]) -> EndRequestBody
    {
        EndRequestBody {
            app_status: BigEndian::read_u32(&data[0..4]),
            protocol_status: data[4],
            reserved: [0; 3],
        }
    }
}

impl Writable for EndRequestBody
{
    fn write(&self) -> Vec<u8>
    {
        let mut data: Vec<u8> = Vec::with_capacity(8);

        let mut buf: [u8; 4] = [0; 4];
        BigEndian::write_u32(&mut buf, self.app_status);
        data.extend_from_slice(&buf);

        data.push(self.protocol_status);
        data.extend_from_slice(&self.reserved);

        data
    }
}

impl Readable for UnknownTypeBody
{
    fn read(data: &[u8]) -> Self
    {
        UnknownTypeBody {
            type_: data[0],
            reserved: [0; 7],
        }
    }
}

impl Writable for UnknownTypeBody
{
    fn write(&self) -> Vec<u8>
    {
        let mut data: Vec<u8> = Vec::with_capacity(8);

        data.push(self.type_);
        data.extend_from_slice(&self.reserved);

        data
    }
}


// ----------------- HTTP implementation -----------------
#[derive(Debug)]
/// HTTP implementation of request
pub struct Request
{
    id: u16,
    role: u16,
    flags: u8,
    headers: HashMap<Vec<u8>, Vec<u8>>,
    readed: bool,
}

impl Request
{
    /// Constructor
    pub fn new(id: u16) -> Request
    {
        Request {
            id: id,
            role: 0,
            flags: 0,
            headers: HashMap::new(),
            readed: false,
        }
    }

    /// Set request options
    pub fn options(&mut self, data: Vec<u8>)
    {
        let begin_request = BeginRequestBody::read(&data[..]);
        self.role = begin_request.role;
        self.flags = begin_request.flags;
    }

    /// Read HTTP-headers pairs
    pub fn param(&mut self, data: Vec<u8>)
    {
        self.headers.extend(ParamFetcher::new(data).parse_param());
    }

    /// Get request id
    pub fn id(&self) -> u16
    {
        self.id
    }

    /// List all headers
    pub fn headers(&self) -> &HashMap<Vec<u8>, Vec<u8>>
    {
        &self.headers
    }

    pub fn mark_readed(&mut self)
    {
        self.readed = true;
    }

    pub fn has_readed(&self) -> bool
    {
        self.readed
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

        for part in data[..].chunks(MAX_LENGTH) {
            result.extend_from_slice(&self.record_header(STDOUT, part.len() as u16));
            result.extend_from_slice(&part);
        }

        // terminate record
        result.extend_from_slice(&self.record_header(STDOUT, 0));
        result.extend_from_slice(&self.end_request());

        result
    }

    /// End request record
    fn end_request(&self) -> Vec<u8>
    {
        let data = EndRequestBody {
                       app_status: 0,
                       protocol_status: REQUEST_COMPLETE,
                       reserved: [0; 3],
                   }
                   .write();

        let mut result: Vec<u8> = self.record_header(END_REQUEST, data.len() as u16);
        result.extend_from_slice(&data);

        result
    }

    /// Get raw header bytes
    fn record_header(&self, type_: u8, length: u16) -> Vec<u8>
    {
        let header = Header {
            version: VERSION_1,
            type_: type_,
            request_id: self.id,
            content_length: length,
            padding_length: 0,
            reserved: [0; 1],
        };

        header.write()
    }

    /// Set some data into response
    pub fn body<T: AsBytes>(&mut self, data: T) -> &mut Response
    {
        self.body.clear();
        self.body.extend_from_slice(data.as_bytes());

        self
    }

    /// Add some HTTP header
    pub fn header<T: AsBytes>(&mut self, key: T, value: T) -> &mut Response
    {
        self.header.insert(Vec::from(key.as_bytes()), Vec::from(value.as_bytes()));

        self
    }

    /// Set custom HTTP status
    pub fn status(&mut self, code: u16) -> &mut Response
    {
        self.header.insert(Vec::from(HTTP_STATUS.as_bytes()),
                           Vec::from(code.to_string().as_bytes()));

        self
    }
}

/// Provide accepting reference to some data types
pub trait AsBytes
{
    /// Must return byte slice
    fn as_bytes(&self) -> &[u8];
}

impl<'a> AsBytes for &'a String
{
    fn as_bytes(&self) -> &[u8]
    {
        String::as_bytes(self)
    }
}

impl<'a> AsBytes for &'a Vec<u8>
{
    fn as_bytes(&self) -> &[u8]
    {
        &self
    }
}

impl<'a> AsBytes for &'a str
{
    fn as_bytes(&self) -> &[u8]
    {
        str::as_bytes(self)
    }
}
