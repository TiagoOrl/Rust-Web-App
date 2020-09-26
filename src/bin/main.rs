use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;

use webServer::{ThreadPool, HTTPHandler, AnalyticsManager};



fn main() {


    let listener = TcpListener::bind("127.0.0.1:5000").unwrap();    // obter Result
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() { // incoming returns an iterator of streams of TcpStreams
        let tcp_stream: TcpStream = stream.unwrap(); // Server handles each stream request

        println!("Novo request");
        pool.execute(|| {
            handle_connection(tcp_stream);
        })
    }
}


fn handle_connection(mut tcp_stream: TcpStream) {
    let mut buffer = [0; 1024];
    tcp_stream.read(&mut buffer).unwrap();
    let http_handler: HTTPHandler = HTTPHandler::new();

    //println!("{}", str::from_utf8(&buffer).unwrap());  // debug

    tcp_stream.write(http_handler.handle_page_request(&buffer).as_bytes()).unwrap();
    tcp_stream.flush().unwrap();
}

