use mio_06::*;
use mio_06::tcp::TcpStream;
use std::io::{Read, Write};
use std::thread;
use std::sync::Arc;
use std::time;

use utils::mock_server;

// Some tokens to allow us to identify which event is for which socket.
const CLIENT: Token = Token(1);

fn main() {
    let addr = Arc::new("127.0.0.1:50052".parse::<std::net::SocketAddr>().unwrap());
    thread::spawn(|| {
        mock_server();
    });
    thread::sleep(time::Duration::from_secs(5));  // dummy method to make sure server is started successfully

    let start = time::Instant::now();
    // Create a poll instance
    let poll = Poll::new().unwrap();
    // Setup the client socket
    let mut sock = TcpStream::connect(&addr).unwrap();

    let request = String::from(
        "GET / HTTP/1.1\r\n"
    );
    sock.write_all(request.as_bytes()).unwrap();
    
    // Register the socket.
    poll.register(&sock, CLIENT, Ready::readable(),PollOpt::edge()).unwrap();

    // Create storage for events.
    let mut events = Events::with_capacity(128);
    
    // Start an event loop.
    loop {
        // Poll Mio for events, blocking until we get an event.
        poll.poll(&mut events, None).unwrap();
        // Process each event.
        for event in events.iter() {
            // println!("[+] RECEIVED EVENT: {:?}", event);
            if event.token() == CLIENT && event.readiness().is_readable() {
                // The socket connected (probably, it could still be a spurious
                // wakeup)
                let mut buf = [0;1024];
                sock.read(&mut buf).unwrap();
                println!("[+] RECEIVED MESSAGE: {}", String::from_utf8_lossy(&buf));
            } 
            println!("[+] elapsed: {:?}", start.elapsed());
        }
    }
}
