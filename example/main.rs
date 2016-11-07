extern crate gfcgi;

//use std::io::Read;
#[derive(Clone, Debug)]
struct Router {}

impl gfcgi::Handler for Router
{
    fn process(&self, request: &mut gfcgi::Request) -> Option<gfcgi::Response>
    {
        let h = request.get_header(b"HTTP_X_TEST");
            println!("{:?}", h);

//        println!("{:?}", String::from_utf8_lossy(reader.get("HTTP_HOST".as_bytes()).unwrap()));
//        let mut buf = Vec::new();
//        reader.read_to_end(&mut buf).unwrap();
//        println!("{:?}", String::from_utf8(buf));

        None
    }
}

fn main()
{
    let client = gfcgi::Client::new("127.0.0.1:4128", Router{});

    client.run(); // spawn tread
    client.run(); // spawn one more
}
