# The FastCGI Rust implementation.

##### Description
*gfcgi* a native Rust library for FastCGI.  
Library is supporting multithreaded socket listener and HTTP-instances multiplexed onto a single connection.  
Documentation is [here](http://gedweb.github.io/rust-gfcgi/gfcgi/ "Documentation").

##### About FastCGI
FastCGI it's great solutions to handling HTTP-requests without overhead. Completely supporting HTTP or HTTPS by any popular web-servers.

##### Planned
- [x] Role
  - [x] responder
  - [ ] filter
  - [ ] authorizer
- [x] Header
  - [ ] get_values
  - [ ] get_values_result
  - [ ] unknown_type
  - [x] begin_request
  - [ ] abort_request
  - [x] end_request
  - [x] params
  - [x] stdin
  - [x] data
  - [x] stdout
  - [ ] stderr

##### Trace
    socket
        └─stream
            ├─connection
            └─handler (request)
                ├─read headers
                ├─optional: read body
                ├─optional: build response
                └─send response