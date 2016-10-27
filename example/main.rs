extern crate gfcgi;

#[derive(Clone, Debug)]
struct Router {}

impl gfcgi::Handler for Router
{
    fn process(&self, reader: &gfcgi::StreamReader)
    {
        println!("{:?}", String::from_utf8_lossy(reader.get("HTTP_HOST".as_bytes()).unwrap()));
    }
}

fn main()
{
    let client = gfcgi::Client::new("127.0.0.1:4128", Router{});

    client.run(); // spawn tread
    client.run(); // spawn one more
}
