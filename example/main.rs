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

    fn process(&self, request: &mut gfcgi::Request, response: &mut gfcgi::Response)
    {
        let h = request.header_utf8(b"HTTP_X_TEST");
        println!("{:?}", h);

        let mut buf = Vec::new();
        request.read_to_end(&mut buf).unwrap();
        println!("{:?}", String::from_utf8(buf));

        response.status(428);
        response.write(&[121u8; 10]).unwrap();
    }
}

fn main()
{
    let client = gfcgi::Client::new("127.0.0.1:4128");

    client.run::<Router>(); // spawn tread
    client.run::<Router>(); // spawn one more
}
