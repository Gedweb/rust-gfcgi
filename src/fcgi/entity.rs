use std::collections::HashMap;

/*
 * Listening socket file number
 */
static LISTENSOCK_FILENO: u8 = 0;

pub struct Header
{
    version: u8,
    type_: u8,
	request_id: u16,
	content_length: u16,
    padding_length: u8,
    reserved: [u8; 1],
}

static MAX_LENGTH: u16 = 0xffff;

/*
 * Number of bytes in a Header.  Future versions of the protocol
 * will not reduce this number.
 */
static HEADER_LEN: u8 = 8;

/*
 * Value for version component of Header
 */
static VERSION_1: u8 = 1;

/*
 * Values for type component of Header
 */
static BEGIN_REQUEST: u8     = 1;
static ABORT_REQUEST: u8     = 2;
static END_REQUEST: u8       = 3;
static PARAMS: u8            = 4;
static STDIN: u8             = 5;
static STDOUT: u8            = 6;
static STDERR: u8            = 7;
static DATA: u8              = 8;
static GET_VALUES: u8        = 9;
static GET_VALUES_RESULT: u8 = 10;
static UNKNOWN_TYPE: u8      = 11;
static MAXTYPE: &'static u8 = &UNKNOWN_TYPE;

/*
 * Value for requestId component of Header
 */
static NULL_REQUEST_ID: u8 = 0;


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
static KEEP_CONN: u8  = 1;

/*
 * Values for role component of BeginRequestBody
 */
static RESPONDER: u8  = 1;
static AUTHORIZER: u8 = 2;
static FILTER: u8     = 3;


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
static REQUEST_COMPLETE: u8 = 0;
static CANT_MPX_CONN: u8    = 1;
static OVERLOADED: u8       = 2;
static UNKNOWN_ROLE: u8     = 3;


/*
 * Variable names for GET_VALUES / GET_VALUES_RESULT records
 */
static MAX_CONNS: &'static str = "MAX_CONNS";
static MAX_REQS: &'static str = "MAX_REQS";
static MPXS_CONNS: &'static str = "MPXS_CONNS";


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








