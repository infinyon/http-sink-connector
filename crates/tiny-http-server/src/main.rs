use std::env;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

extern crate tiny_http;

#[derive(Clone, Debug, Default)]
pub struct RequestData {
    #[allow(dead_code)]
    payload: String,
    #[allow(dead_code)]
    user_agent: String,
}

#[derive(Clone, Debug, Default)]
struct State {
    counter: Arc<Mutex<AtomicU32>>,
    payload: Arc<Mutex<Vec<RequestData>>>,
}

impl State {
    fn get_count(&self) -> u32 {
        self.counter.lock().unwrap().get_mut().to_owned()
    }

    fn get_payload(&self) -> Vec<RequestData> {
        self.payload.lock().unwrap().to_owned()
    }

    fn count(&self) -> u32 {
        self.counter.lock().unwrap().fetch_add(1, Ordering::SeqCst)
    }

    fn append_payload(&self, data: RequestData) {
        self.payload.lock().unwrap().push(data);
    }
}

struct FileLogger {
    file: File,
}

impl FileLogger {
    fn new(name: &str) -> Self {
        let file_name = Self::final_name(name);
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_name)
            .expect("Failed to create file");

        println!("Created logs file with name: {}", file_name);
        Self { file }
    }

    fn final_name(s: &str) -> String {
        if s.is_empty() {
            println!("Using default name: log.txt");
            return String::from("log.txt");
        }

        String::from(s)
    }

    fn write_log(&mut self, counter: u32, payload: Vec<RequestData>) {
        writeln!(self.file, "Counter: {}\nPayloads: \n{:?}", counter, payload)
            .expect("Failed to write to file");
    }
}

fn main() {
    use tiny_http::{Response, Server};

    let logger_name = env::var("LOGGER_FILENAME").unwrap_or(String::from("log.txt"));
    let state = State::default();
    let server = Server::http("127.0.0.1:8080").unwrap();
    let mut file_logger = FileLogger::new(&logger_name);

    for mut request in server.incoming_requests() {
        let mut content = String::new();
        request.as_reader().read_to_string(&mut content).unwrap();
        let user_agent = request
            .headers()
            .iter()
            .find(|h| h.field.equiv("user-agent"));

        if !content.is_empty() {
            let data = RequestData {
                payload: content.clone(),
                user_agent: user_agent.map(|h| h.value.to_string()).unwrap_or_default(),
            };

            state.append_payload(data);
        }

        state.count();

        println!("Content: {:?}", content);

        let response = Response::from_string("Ok");
        request.respond(response).expect("Replied");

        file_logger.write_log(state.get_count(), state.get_payload());
    }
}
