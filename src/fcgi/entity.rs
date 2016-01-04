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
    pub reserved: [u8; 1],
}

impl Header
{
    pub fn new () -> Header
    {
        Header {
            version: 0,
            type_: 0,
        	request_id: 0,
        	content_length: 0,
            padding_length: 0,
            reserved: [0; 1],
        }
    }
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
pub const BEGIN_REQUEST: u8     = 1;
pub const ABORT_REQUEST: u8     = 2;
pub const END_REQUEST: u8       = 3;
pub const PARAMS: u8            = 4;
pub const STDIN: u8             = 5;
pub const STDOUT: u8            = 6;
pub const STDERR: u8            = 7;
pub const DATA: u8              = 8;
pub const GET_VALUES: u8        = 9;
pub const GET_VALUES_RESULT: u8 = 10;
pub const UNKNOWN_TYPE: u8      = 11;
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

pub struct BeginRequestRecord
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


pub struct EndRequestBody
{
    app_status: u32,
    protocol_status: u8,
    reserved: [u8; 3],
}

pub struct EndRequestRecord
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


pub struct UnknownTypeBody
{
    type_: u8,    
    reserved: [u8; 7],
}

pub struct UnknownTypeRecord
{
    header: Header,
    body: UnknownTypeBody,
}

/*
 * Request message
 */
pub struct Request
{
    headers: HashMap<String, String>,
    body: String,
}








