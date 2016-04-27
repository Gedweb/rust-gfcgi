/* ----------------- entity ----------------- */
use std::collections::HashMap;
use std::sync::mpsc;

/*
 * Listening socket file number
 */
pub const LISTENSOCK_FILENO: u8 = 0;

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

pub const MAX_LENGTH: usize = 0xffff;

/*
 * Number of bytes in a Header.  Future versions of the protocol
 * will not reduce this number.
 */
pub const HEADER_LEN: usize = 8;

/*
 * Value for version component of Header
 */
pub const VERSION_1: u8 = 1;

/*
 * Values for type component of Header
 */
pub const BEGIN_REQUEST: u8     = 1;  // WS
pub const ABORT_REQUEST: u8     = 2;  // WS
pub const END_REQUEST: u8       = 3;
pub const PARAMS: u8            = 4;  // WS | stream
pub const STDIN: u8             = 5;  // WS | stream
pub const STDOUT: u8            = 6;  //    | stream
pub const STDERR: u8            = 7;  //    | stream
pub const DATA: u8              = 8;  // WS | stream
pub const GET_VALUES: u8        = 9;  // WS | management
pub const GET_VALUES_RESULT: u8 = 10; //    | management
pub const UNKNOWN_TYPE: u8      = 11; //    | management
pub const MAXTYPE: &'static u8  = &UNKNOWN_TYPE;

/*
 * Value for requestId component of Header
 */
pub const NULL_REQUEST_ID: u8 = 0;


pub struct BeginRequestBody
{
    role: u16,
    flags: u8,
    reserved: [u8; 5],
}

struct BeginRequestRecord
{
    header: Header,
    body: BeginRequestBody,
}

/*
 * Mask for flags component of BeginRequestBody
 */
pub const KEEP_CONN: u8  = 1;

/*
 * Values for role component of BeginRequestBody
 */
pub const RESPONDER: u8  = 1;
pub const AUTHORIZER: u8 = 2;
pub const FILTER: u8     = 3;


struct EndRequestBody
{
    app_status: u32,
    protocol_status: u8,
    reserved: [u8; 3],
}

struct EndRequestRecord
{
    header: Header,
    body: EndRequestBody,
}

/*
 * Values for protocolStatus component of EndRequestBody
 */
pub const REQUEST_COMPLETE: u8 = 0;
pub const CANT_MPX_CONN: u8    = 1;
pub const OVERLOADED: u8       = 2;
pub const UNKNOWN_ROLE: u8     = 3;


/*
 * Variable names for GET_VALUES / GET_VALUES_RESULT records
 */
pub const MAX_CONNS: &'static str = "MAX_CONNS";
pub const MAX_REQS: &'static str = "MAX_REQS";
pub const MPXS_CONNS: &'static str = "MPXS_CONNS";


struct UnknownTypeBody
{
    type_: u8,
    reserved: [u8; 7],
}

struct UnknownTypeRecord
{
    pub header: Header,
    pub body: UnknownTypeBody,
}

/* ----------------- repository ----------------- */

extern crate byteorder;
use self::byteorder::{ByteOrder, BigEndian};

pub trait Readable {
    fn read(data: &[u8]) -> Self;
}

pub trait Writable {
    fn write(&self) -> Vec<u8>;
}

/* ----------------- implimentation ----------------- */

impl Readable for Header
{
    fn read(data: &[u8]) -> Header
    {
        Header {
            version: data[0],
            type_: data[1],
            request_id: BigEndian::read_u16(&data[2 .. 4]),
            content_length: BigEndian::read_u16(&data[4 .. 6]),
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
            role: BigEndian::read_u16(&data[0 .. 2]),
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
            app_status: BigEndian::read_u32(&data[0 .. 4]),
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


/* ----------------- HTTP implimentation ----------------- */
#[derive(Debug)]
pub struct Request
{
    pub id: u16,
    pub role: u16,
    pub flags: u8,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    stream_tx: mpsc::Sender<Response>,
}

impl Request
{
    pub fn new(id: u16, tx: mpsc::Sender<Response>) -> Request
    {
        Request {
            id: id,
            role: 0,
            flags: 0,
            headers: HashMap::new(),
            body: Vec::new(),
            stream_tx: tx,
        }
    }

    pub fn add_record(&mut self, header: &Header, body_data: Vec<u8>)
    {
        match header.type_ {
            BEGIN_REQUEST => self.options(body_data),
            PARAMS => self.param(body_data),
            STDIN => self.stdin(body_data),
            DATA => self.stdin(body_data),
            _ => panic!("Undeclarated fastcgi header"),
        };
    }

    fn options(&mut self, data: Vec<u8>)
    {
        let begin_request = BeginRequestBody::read(&data[..]);
        self.role = begin_request.role;
        self.flags = begin_request.flags;
    }

    fn param(&mut self, data: Vec<u8>)
    {
        self.headers.extend(ParamFetcher::new(data).parse_param());
    }

    fn stdin(&mut self, data: Vec<u8>)
    {
        self.body.extend_from_slice(&data);
    }

    pub fn get_id(&self) -> u16
    {
        self.id
    }

    pub fn body(&self) -> &Vec<u8>
    {
        &self.body
    }

    pub fn body_string(&self) -> String
    {
        String::from_utf8_lossy(&self.body).into_owned()
    }

    pub fn reply(&self, mut response: Response)
    {
        response.set_id(self.id);
        self.stream_tx.send(response).unwrap();
    }
}

/*
 * Split key-value param pairs
 */
struct ParamFetcher
{
    data: Vec<u8>,
    pos: usize,
}

impl ParamFetcher
{
    fn new(data: Vec<u8>) -> Self
    {
        ParamFetcher {
            data: data,
            pos: 0,
        }
    }

    fn parse_param(&mut self) -> HashMap<String, String>
    {
        let mut param: HashMap<String, String> = HashMap::new();

        let data_length: usize = self.data.len();

        while data_length > self.pos {

            let key_length: usize = self.param_length();
            let value_length: usize = self.param_length();

            let key: String = String::from_utf8_lossy(&self.data[self.pos .. self.pos+key_length]).into_owned();
            self.pos += key_length;

            let value: String = String::from_utf8_lossy(&self.data[self.pos .. self.pos+value_length]).into_owned();
            self.pos += value_length;

            param.insert(key, value);
        }

        param
    }

    fn param_length(&mut self) -> usize
    {
        let mut length: usize = self.data[self.pos] as usize;

        if (length >> 7) == 1 {

            self.data[self.pos] = self.data[self.pos] & 0x7F;
            length = BigEndian::read_u32(&self.data[self.pos .. (self.pos+4)]) as usize;

            self.pos += 4;
        } else {
            self.pos += 1;
        }

        length
    }
}

/*
 * Response for request
 */

const HTTP_STATUS: &'static str = "Status";
const HTTP_LINE: &'static str = "\r\n";

#[derive(Debug)]
pub struct Response
{
    id: u16,
    header: HashMap<Vec<u8>, Vec<u8>>,
    body: Vec<u8>,
}

impl Response
{
    pub fn new() -> Response
    {
        let mut header = HashMap::new();
        header.insert(Vec::from(HTTP_STATUS.as_bytes()), Vec::from("200".as_bytes()));

        Response {
            id: 0,
            header: header,
            body: Vec::new(),
        }
    }

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

    fn end_request(&self) -> Vec<u8>
    {
        let data = EndRequestBody {
            app_status: 0,
            protocol_status: REQUEST_COMPLETE,
            reserved: [0; 3],
        }.write();

        let mut result: Vec<u8> = self.record_header(END_REQUEST, data.len() as u16);
        result.extend_from_slice(&data);

        result
    }

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

    fn set_id(&mut self, id: u16)
    {
        self.id = id;
    }

    pub fn get_id(&self) -> &u16
    {
        &self.id
    }

    pub fn set_body<T: AsBytes>(&mut self, data: T) -> &mut Response
    {
        self.body.extend_from_slice(data.as_bytes());

        self
    }

    pub fn set_header<T: AsBytes>(&mut self, key: T, value: T) -> &mut Response
    {
        self.header.insert(Vec::from(key.as_bytes()), Vec::from(value.as_bytes()));

        self
    }

    pub fn set_status(&mut self, code: u16) -> &mut Response
    {
        self.header.insert(Vec::from(HTTP_STATUS.as_bytes()), Vec::from(code.to_string().as_bytes()));

        self
    }
}

pub trait AsBytes
{
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
