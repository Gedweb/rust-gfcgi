extern crate fcgi;

use fcgi::fcgi::client;

fn main() {
    client::Listener::new("localhost:4128").run();
}