extern crate gfcgi;

use std::io::{Read, Write};
use std::thread;
use std::sync::Arc;

use std::net::TcpListener;

struct Router();

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

fn main()
{
    let listener = TcpListener::bind("0.0.0.0:4128").expect("Bind address");

    // run listeners
    for _ in 0..5 {
        let listener = listener.try_clone().expect("Clone listener");
        thread::spawn(move || {
            gfcgi::listen(listener.incoming(), Arc::new(Router()));
        });
    }

    thread::park();
}
