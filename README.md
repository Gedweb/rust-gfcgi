# The FastCGI Rust implementation.

[![Build Status](https://travis-ci.org/Gedweb/rust-gfcgi.svg?branch=master)](https://travis-ci.org/Gedweb/rust-gfcgi) [![docs.rs](https://docs.rs/gfcgi/badge.svg)](https://docs.rs/gfcgi) [![Cargo](https://img.shields.io/crates/v/gfcgi.svg)](https://crates.io/crates/gfcgi) [![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

##### Description
*gfcgi* a native Rust library for FastCGI.  
Library is supporting multithreaded socket listener and HTTP-instances multiplexed onto a single connection.

##### About FastCGI
FastCGI it's great solutions to handling HTTP-requests without overhead. Completely supporting HTTP or HTTPS by any popular web-servers. 

[Specification](doc/fcgi-spec.md)

##### Planned
- [x] Role
  - [x] responder
  - [ ] filter
  - [ ] authorizer
- [x] Header
  - [ ] get_values
  - [ ] get_values_result
  - [x] unknown_type
  - [x] begin_request
  - [x] abort_request
  - [x] end_request
  - [x] params
  - [x] stdin
  - [ ] data
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