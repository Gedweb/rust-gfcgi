extern crate gfcgi;

use std::thread;

fn main() {

    let client = gfcgi::Client::bind("127.0.0.1:4128");

    client.run(); // spawn tread
    client.run(); // spawn one more

    for request in client {

        thread::spawn(move || {
            let mut response = gfcgi::model::Response::new();

            // resend
            response.body(request.body_raw());
            // append header
            response.header("Content-Type", "text/plain; charset=utf-8");

            request.reply(response);
        });
    }
}
