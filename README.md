# The FastCGI Rust implementation.

[![Build Status](https://travis-ci.org/Gedweb/rust-gfcgi.svg)](https://travis-ci.org/Gedweb/rust-gfcgi) [![docs.rs](https://docs.rs/gfcgi/badge.svg)](https://docs.rs/gfcgi) [![Cargo](https://img.shields.io/crates/v/gfcgi.svg)](https://crates.io/crates/gfcgi) [![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

#### Description
*gfcgi* a native Rust library for FastCGI.  
Library is supporting multithreaded socket listener and HTTP-instances multiplexed onto a single connection.

#### About FastCGI
FastCGI it's great solutions to handling HTTP-requests without overhead. Completely supporting HTTP or HTTPS by any popular web-servers. 

[Specification](doc/fcgi-spec.md) 

#### Example
Import the library within your code.
```rust
    extern crate gfcgi;
    
    use std::io::{Read, Write}; 
    use std::thread;
```
Some your router struct
```rust
    #[derive(Clone)]
    struct Router;
        
    impl Router
    {
        fn new() -> Self
        { 
            Router{}
        }
    }
```
Implement [`gfcgi::Handler`](https://docs.rs/gfcgi/0.4.3/gfcgi/trait.Handler.html) trait for your router, all code in `process` method is optional
```rust
    impl gfcgi::Handler for Router
    {
        fn process(&self, request: &mut gfcgi::Request, response: &mut gfcgi::Response)
        {
            // request headers can available any time
            let host = request.header_utf8(b"HTTP_HOST")
                .unwrap_or("not provided")
                .to_owned()
            ;
    
            // read content before start response if you want to use it
            let mut buf = Vec::new();
            request.read_to_end(&mut buf).expect("read body");
    
            // set status and header
            response
                .status(200)
                .header_utf8("Content-type", "text/plain");
    
            // send content
            response.write(
                format!("hello `{}`", host).as_bytes()
            ).expect("send body");
        }
    }
```
Now run [`listener`](https://docs.rs/gfcgi/0.4.3/gfcgi/struct.Client.html), you can spawn thread if set `spawn` feature in `Cargo.toml`
```rust
fn main()
{
    let client = gfcgi::Client::new("127.0.0.1:4128");

    // run listeners
    for _ in 0..5 {
        let client = client.clone();
        thread::spawn(move || {
            client.run(Router::new());
        });
    }

    // keep main process
    thread::park();
}
```
#### Planned
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

#### Trace
    socket
        stream
            connection
            handler
                request
                | → read headers
                | → [read body]
                response
                | ← write headers
                | ← [write body]