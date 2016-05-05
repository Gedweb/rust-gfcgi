initSidebarItems({"constant":[["ABORT_REQUEST","type component of Header # Request A Web server aborts a FastCGI request when an HTTP client closes its transport connection while the FastCGI request is running on behalf of that client"],["AUTHORIZER","FastCGI role authorized/unauthorized decision"],["BEGIN_REQUEST","type component of Header # Request The Web server sends a FCGI_BEGIN_REQUEST record to start a request"],["CANT_MPX_CONN","protocol_status component of EndRequestBody"],["DATA","type component of Header # Request Byte Stream"],["END_REQUEST","type component of Header # Response The application sends a FCGI_END_REQUEST record to terminate a request"],["FILTER","FastCGI role extra stream of data from a file"],["GET_VALUES","type component of Header # Request The Web server can query specific variables within the application The application receives."],["GET_VALUES_RESULT","type component of Header # Response The Web server can query specific variables within the application. The application responds."],["HEADER_LEN","Number of bytes in a Header."],["KEEP_CONN","Mask for flags component of BeginRequestBody"],["LISTENSOCK_FILENO","Listening socket file number"],["MAX_CONNS","Names for GET_VALUES / GET_VALUES_RESULT records."],["MAX_LENGTH","Maximum length per record"],["MAX_REQS","Names for GET_VALUES / GET_VALUES_RESULT records."],["MPXS_CONNS","Names for GET_VALUES / GET_VALUES_RESULT records."],["NULL_REQUEST_ID","Default request id component of Header"],["OVERLOADED","protocol_status component of EndRequestBody"],["PARAMS","type component of Header # Request Receive name-value pairs from the Web server to the application"],["REQUEST_COMPLETE","protocol_status component of EndRequestBody"],["RESPONDER","FastCGI role emulated CGI/1.1 program"],["STDERR","type component of Header # Response Byte Stream"],["STDIN","type component of Header # Request Byte Stream"],["STDOUT","type component of Header # Response Byte Stream"],["UNKNOWN_ROLE","protocol_status component of EndRequestBody"],["UNKNOWN_TYPE","type component of Header"],["VERSION_1","version component of Header"]],"struct":[["Header","FCGI record header"],["Request","HTTP implementation of request"],["Response","HTTP implementation of response"]],"trait":[["AsBytes","Provide accepting reference to some data types"],["Readable",""],["Writable",""]]});