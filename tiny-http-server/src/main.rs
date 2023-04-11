extern crate tiny_http;

fn main() {
    use tiny_http::{Response, Server};

    let server = Server::http("127.0.0.1:8080").unwrap();

    for mut request in server.incoming_requests() {
        println!(
            "Request: {:?}, url: {:?}, headers: {:?}",
            request.method(),
            request.url(),
            request.headers()
        );

        let mut content = String::new();
        request.as_reader().read_to_string(&mut content).unwrap();

        println!("Content: {:?}", content);

        let response = Response::from_string("Ok");
        request.respond(response).expect("Replied");
    }
}