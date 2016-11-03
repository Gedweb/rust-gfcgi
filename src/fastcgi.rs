//! Contain constants and models for fcgi data records.

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
    pub reserved: [u8; 1],
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
pub struct BeginRequestBody
{
    pub role: u16,
    pub flags: u8,
    pub reserved: [u8; 5],
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
pub struct EndRequestBody
{
    pub app_status: u32,
    pub protocol_status: u8,
    pub reserved: [u8; 3],
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
    pub type_: u8,
    pub reserved: [u8; 7],
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
