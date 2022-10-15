use std::error::Error;
use std::{thread, time};
use utils::{init_log, mock_server};
use std::io::{Write, Read};

use mio_07::net::TcpStream;
use mio_07::{Events, Interest, Poll, Token};

// Some tokens to allow us to identify which event is for which socket.
const CLIENT: Token = Token(1);

fn main() -> Result<(), Box<dyn Error>> {
    init_log();
    thread::spawn(|| {
        mock_server();
    });
    thread::sleep(time::Duration::from_secs(5));  // dummy method to make sure server is started successfully

    let start = time::Instant::now();
    // Create a poll instance.
    let mut poll = Poll::new()?;
    // Create storage for events.
    let mut events = Events::with_capacity(128);

    let addr = "127.0.0.1:50052".parse()?;

    // Setup the client socket.
    let mut client = TcpStream::connect(addr)?;
    // Register the socket.
    poll.registry()
        .register(&mut client, CLIENT, Interest::READABLE | Interest::WRITABLE)?;

    let request = String::from(
        "GET / HTTP/1.1\r\n"
    );
    client.write_all(request.as_bytes()).unwrap();

    // Start an event loop.
    loop {
        // Poll Mio for events, blocking until we get an event.
        poll.poll(&mut events, None)?;

        // Process each event.
        for event in events.iter() {    // the iteration API was changed to return a reference to Event rather than making a copy
            // We can use the token we previously provided to `register` to
            // determine for which socket the event is.
            // println!("[+] RECEIVED EVENT: {:?}", event);
            if event.token() == CLIENT && event.is_readable() {
                // The socket connected (probably, it could still be a spurious
                // wakeup)
                let mut buf = [0;1024];
                client.read(&mut buf).unwrap();
                println!("[+] RECEIVED MESSAGE: {}", String::from_utf8_lossy(&buf));
            } 
            println!("[+] elapsed: {:?}", start.elapsed());
        }
    }
}