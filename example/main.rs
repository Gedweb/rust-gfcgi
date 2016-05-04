extern crate gfcgi;

use std::thread;

fn main() {

    let client = gfcgi::Client::bind("127.0.0.1:80");

    client.run(); // spawn tread
    client.run(); // spawn one more

    for request in client {

        thread::spawn(move || {

            let mut response = gfcgi::model::Response::new();

            response.body(request.body_raw()); // send back recieved
            response.body(&request.header("HTTP_HOST").unwrap());
            response.status(201); // not required
            response.header("Content-Type", "text/plain; charset=utf-8");
            request.reply(response);
        });
    }
}
