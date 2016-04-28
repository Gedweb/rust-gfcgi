use std::net::TcpListener;

extern crate gfcgi;


fn main() {

    let listener = TcpListener::bind("127.0.0.1:4128").unwrap();
    let client = gfcgi::Client::new(listener);

    client.run();

    for request in client {
        println!("{:?}", request);
        let mut response = gfcgi::model::Response::new();

        response.set_body("Accepted");
        response.set_status(200);
        request.reply(response);
    }

}

