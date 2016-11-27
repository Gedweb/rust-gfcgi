extern crate gfcgi;

use std::io::{Read, Write};
#[derive(Clone, Debug)]
struct Router;

impl gfcgi::Handler for Router
{
    fn new() -> Self
    {
        Router {

        }
    }

    fn process(&self, fcgi: &mut gfcgi::HttpPair)
    {
        // get a header
        let h = fcgi.request().header_utf8(b"HTTP_X_TEST");
        println!("{:?}", h);

        // read content
        let mut buf = Vec::new();
        fcgi.request().read_to_end(&mut buf).unwrap();
        println!("{:?}", String::from_utf8(buf));

        // set header
        fcgi.response().header_utf8("Content-type", "text/plain");

        // send content
        fcgi.response().write(b"hello world!").expect("send body");

    }
}

fn main()
{
    let client = gfcgi::Client::new("127.0.0.1:4128");

    client.run::<Router>(); // spawn tread
//    client.run::<Router>(); // spawn one more
}
