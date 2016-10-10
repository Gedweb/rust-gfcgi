extern crate gfcgi;

#[derive(Clone, Debug)]
struct Router {}

impl gfcgi::Handler for Router
{
    fn process(&self, request: gfcgi::Request)
    {
        for (key, val) in request.headers() {
            println!("{}: {}", String::from_utf8_lossy(key), String::from_utf8_lossy(val));
        }
    }
}

fn main()
{
    let client = gfcgi::Client::new("127.0.0.1:4128", Router{});

    client.run(); // spawn tread
    client.run(); // spawn one more
}
