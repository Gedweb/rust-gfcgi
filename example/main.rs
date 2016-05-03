extern crate gfcgi;

use std::net::TcpListener;
use std::thread;

fn main() {

    let listener = TcpListener::bind("127.0.0.1:4128").unwrap();
    let client = gfcgi::Client::new(listener);

    client.run(); // spawn tread
    client.run(); // spawn one more

    for request in client {

        thread::spawn(move || {

            let mut response = gfcgi::model::Response::new();

            response.body(request.body()); // send back recieved
            response.body(request.header("HTTP_HOST").unwrap());
            response.status(201); // not required
            response.header("Content-Type", "text/plain; charset=utf-8");
            request.reply(response);
        });
    }

}

