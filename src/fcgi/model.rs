/*
 * Entity
 */

use std::collections::HashMap;

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
    pub role: u16,
    pub flags: u8,
    pub reserved: [u8; 5],
}

pub struct BeginRequestRecord
{
    pub header: Header,
    pub body: BeginRequestBody,
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


pub struct EndRequestBody
{
    pub app_status: u32,
    pub protocol_status: u8,
    pub reserved: [u8; 3],
}

pub struct EndRequestRecord
{
    pub header: Header,
    pub body: EndRequestBody,
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


pub struct UnknownTypeBody
{
    type_: u8,    
    reserved: [u8; 7],
}

pub struct UnknownTypeRecord
{
    pub header: Header,
    pub body: UnknownTypeBody,
}

/*
 * Request message
 */
#[derive(Debug)]
pub struct Request
{
    headers: HashMap<String, String>,
    body: String,
}

/*
 * Repository
 */
extern crate byteorder;
use self::byteorder::{ByteOrder, BigEndian, LittleEndian};

use std::iter::Extend;

impl Header
{
    pub fn read(data: &[u8]) -> Header
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
    
    pub fn write(&self) -> Vec<u8>
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


impl BeginRequestBody
{
    pub fn read(data: &[u8]) -> BeginRequestBody 
    {
        BeginRequestBody {
            role: BigEndian::read_u16(&data[0 .. 2]),
            flags: data[2],
            reserved: [0; 5],
        }
    }
    
    pub fn write(&self) -> Vec<u8>
    {
        let mut data: Vec<u8> = Vec::new();
        
        BigEndian::write_u16(&mut data, self.role);
        data.push(self.flags);
        data.extend_from_slice(&self.reserved);
        
        data
    }
}


impl EndRequestBody
{
    pub fn read(data: &[u8]) -> EndRequestBody
    {
        EndRequestBody {
            app_status: BigEndian::read_u32(&data[0 .. 4]),
            protocol_status: data[4],
            reserved: [0; 3],
        }
	}
    
    pub fn write(&self) -> Vec<u8>
    {
        let mut data: Vec<u8> = Vec::new();
        
        BigEndian::write_u32(&mut data, self.app_status);
        data.push(self.protocol_status);
        data.extend_from_slice(&self.reserved);
        
        data
    }
}

impl UnknownTypeBody
{
    pub fn read(data: &[u8]) -> Self
    {
        UnknownTypeBody {
            type_: data[0],    
            reserved: [0; 7],
        }
    }
    
    pub fn write(&self) -> Vec<u8>
    {
        let mut data: Vec<u8> = Vec::new();
        
        data.push(self.type_);
        data.extend_from_slice(&self.reserved);
        
        data
    }
}

impl Request
{
    pub fn new() -> Self
    {
        Request {
            headers: HashMap::new(),
            body: String::new(),            
        }
    }
    
    pub fn add_param(&mut self, data: Vec<u8>)
    {
        self.headers.extend(ParamFetcher::new(data).parse_param());
    }
    
    pub fn add_body(&mut self, data: Vec<u8>)
    {
        self.body = self.body.to_string() + &String::from_utf8(data).unwrap();
    }
}


struct ParamFetcher
{
    data: Vec<u8>,
    pos: usize,
}

impl ParamFetcher
{
    pub fn new(data: Vec<u8>) -> Self
    {
        ParamFetcher {
            data: data,
            pos: 0,
        }
    }
    
    pub fn parse_param(&mut self) -> HashMap<String, String>
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























