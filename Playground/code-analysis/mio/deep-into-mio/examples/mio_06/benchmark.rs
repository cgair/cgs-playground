use utils::timed;
use mio_06::{
    Token, Poll, Events, Ready, PollOpt,
    net::TcpStream,
};
use std::io::{Read, Write};

const CLIENT: Token = Token(1);

fn poll_bench() {
    let addr = "127.0.0.1:50052".parse::<std::net::SocketAddr>().unwrap();
    let mut sock = TcpStream::connect(&addr).unwrap();

    let poll = Poll::new().unwrap();

    let request = String::from(
        "GET / HTTP/1.1\r\n"
    );
    sock.write_all(request.as_bytes()).unwrap();

    poll.register(&sock, CLIENT, Ready::readable(),PollOpt::edge()).unwrap();
    let mut events = Events::with_capacity(128);
    loop {
        poll.poll(&mut events, None).unwrap();
        for event in events.iter() {
            if event.token() == CLIENT && event.readiness().is_readable() {
                let mut buf = [0;1024];
                sock.read(&mut buf).unwrap();
                println!("[+] Poll once RCVD: {}", String::from_utf8_lossy(&buf));
                return;
            } 
        }
    }
}

fn main() {
    timed("poll", poll_bench);
}