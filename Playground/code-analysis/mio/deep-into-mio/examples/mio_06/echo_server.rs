#![allow(deprecated)]
use mio_06::{
    net::{TcpListener, TcpStream},
    Poll, Token, PollOpt, Ready, Events,
    
};
use bytes::{ByteBuf, MutByteBuf, Buf};
use utils::{
    init_log, localhost,
    TryRead, TryWrite,
};
use slab::Slab;
use log::debug;
use std::{
    io, thread, sync::mpsc
};

const SERVER: Token = Token(10_000_000);
const CLIENT: Token = Token(10_000_001);

struct EchoServer {
    sock: TcpListener,
    conns: Slab<EchoConn>
}

impl EchoServer {
    fn accept(&mut self, poll: &Poll) -> io::Result<()> {
        debug!("server accepting socket");
        let sock = self.sock.accept().unwrap().0;
        let conn = EchoConn::new(sock);
        let tok = self.conns.insert(conn);

        // Register the connection
        self.conns[tok].token = Some(Token(tok));
        poll.register(&self.conns[tok].sock, Token(tok), Ready::readable(),
                                PollOpt::edge() | PollOpt::oneshot())
            .ok().expect("could not register socket with event queue");
        
        Ok(())
    }

    fn conn_readable(&mut self, poll: &Poll, tok: Token) -> io::Result<()> {
        debug!("server conn readable; tok={:?}", tok);
        self.conn(tok).readable(poll)
    }

    fn conn_writable(&mut self, poll: &Poll, tok: Token) -> io::Result<()> {
        debug!("server conn writable; tok={:?}", tok);
        self.conn(tok).writable(poll)
    }

    fn conn<'a>(&'a mut self, tok: Token) -> &'a mut EchoConn {
        &mut self.conns[tok.0]
    }
}

struct EchoConn {
    sock: TcpStream,
    buf: Option<ByteBuf>,
    mut_buf: Option<MutByteBuf>,
    token: Option<Token>,
    interest: Ready,
}

impl EchoConn {
    fn new(sock: TcpStream) -> Self {
        EchoConn {
            sock,
            buf: None,
            mut_buf: Some(ByteBuf::mut_with_capacity(2048)),
            token: None,
            interest: Ready::hup()
        }
    }

    fn readable(&mut self, poll: &Poll) -> io::Result<()> {
        let mut buf = self.mut_buf.take().unwrap();
        match self.sock.try_read_buf(&mut buf) {
            Ok(None) => {
                debug!("CONN : spurious read wakeup");
                self.mut_buf = Some(buf);
            },
            Ok(Some(r)) => {
                debug!("CONN : we read {} bytes!", r);

                // prepare to provide this to writable
                self.buf = Some(buf.flip());

                self.interest.insert(Ready::readable());
                self.interest.remove(Ready::writable());
            },
            Err(e) => {
                debug!("not implemented; client err={:?}", e);
                self.interest.remove(Ready::readable());
            }
        }
        assert!(self.interest.is_readable() || self.interest.is_writable(), "actual={:?}", self.interest);
        poll.reregister(&self.sock, self.token.unwrap(), self.interest,
                              PollOpt::edge())
    }

    fn writable(&mut self, poll: &Poll) -> io::Result<()> {
        let mut buf = self.buf.take().unwrap();

        match self.sock.try_write_buf(&mut buf) {
            Ok(None) => {
                debug!("client flushing buf; WOULDBLOCK");

                self.buf = Some(buf);
                self.interest.insert(Ready::writable());
            },
            Ok(Some(r)) => {
                debug!("CONN : we wrote {} bytes!", r);

                self.mut_buf = Some(buf.flip());

                self.interest.insert(Ready::readable());
                self.interest.remove(Ready::writable());
            },
            Err(e) => debug!("not implemented; client err={:?}", e),
        }

        assert!(self.interest.is_readable() || self.interest.is_writable(), "actual={:?}", self.interest);
        poll.reregister(&self.sock, self.token.unwrap(), self.interest,
                              PollOpt::edge() | PollOpt::oneshot())
    }
}

struct EchoClient {
    sock: TcpStream,
    rx: mpsc::Receiver<ByteBuf>,
    mut_buf: Option<MutByteBuf>,
    token: Token,
    interest: Ready
}

// Sends a message and expects to receive the same exact message, one at a time
impl EchoClient {
    fn new(sock: TcpStream, tok: Token, rx: mpsc::Receiver<ByteBuf>) -> Self {
        EchoClient {
            sock,
            rx,
            mut_buf: Some(ByteBuf::mut_with_capacity(2048)),
            token: tok,
            interest: Ready::none()
        }
    }

    fn readable(&mut self, poll: &Poll) -> io::Result<()> {
        debug!("client socket readable");
        let mut buf = self.mut_buf.take().unwrap();

        match self.sock.try_read_buf(&mut buf) {
            Ok(None) => {
                debug!("CLIENT : spurious read wakeup");
                self.mut_buf = Some(buf);
            },
            Ok(Some(r)) => {
                debug!("CLIENT : We read {} bytes!", r);

                // prepare for reading
                let mut buf = buf.flip();

                while buf.has_remaining() {
                    let actual = buf.read_byte().unwrap();

                    debug!("actual rcvd={}", actual);
                }

                self.mut_buf = Some(buf.flip());

                self.interest.remove(Ready::readable());
            },
            Err(e) => {
                panic!("not implemented; client err={:?}", e);
            }
        }

        if !self.interest.is_none() {
            assert!(self.interest.is_readable() || self.interest.is_writable(), "actual={:?}", self.interest);
            poll.reregister(&self.sock, self.token, self.interest,
                                       PollOpt::edge() | PollOpt::oneshot()).unwrap();
        }

        Ok(())
    }

    fn writable(&mut self, poll: &Poll) -> io::Result<()> {
        debug!("client socket writable");
        while let Ok(mut buf) = self.rx.recv() {
            match self.sock.try_write_buf(&mut buf) {
                Ok(None) => {
                    debug!("client flushing buf; WOULDBLOCK");
                    self.interest.insert(Ready::writable());
                    break;
                }
                Ok(Some(r)) => {
                    debug!("CLIENT : we wrote {} bytes!", r);
                    self.interest.insert(Ready::readable());
                    self.interest.remove(Ready::writable());
                    break;
                }
                Err(e) => panic!("not implemented; client err={:?}", e)
            }
        }

        assert!(self.interest.is_readable() || self.interest.is_writable(), "actual={:?}", self.interest);
        poll.reregister(&self.sock, self.token, self.interest,
            PollOpt::edge() | PollOpt::oneshot())
    }
}

struct Echo {
    server: EchoServer,
    client: EchoClient,
}

impl Echo {
    fn new(srv: TcpListener, client: TcpStream, rx: mpsc::Receiver<ByteBuf>) -> Self {
        Echo {
            server: EchoServer {
                sock: srv,
                conns: Slab::with_capacity(128)
            },
            client: EchoClient::new(client, CLIENT, rx)
        }
    }
}

fn main() {
    init_log();
    // Create a poll instance
    let poll = Poll::new().unwrap();

    debug!("Starting TEST_ECHO_SERVER");
    let addr = localhost();
    let srv = TcpListener::bind(&addr).unwrap();

    log::info!("listen for connections");
    poll.register(&srv, SERVER, Ready::readable(), PollOpt::edge() | PollOpt::oneshot()).unwrap();

    let client = TcpStream::connect(&addr).unwrap();
    // Connect to the server
    poll.register(&client, CLIENT, Ready::writable(), PollOpt::edge() | PollOpt::oneshot()).unwrap();

    // let (s_tx, s_rx) = mpsc::channel();
    let (c_tx, c_rx) = mpsc::channel();
    let mut echo = Echo::new(srv, client, c_rx);

    /* CLIENT: provide msgs to be send through channel */
    thread::spawn(move || {
        let msg = ByteBuf::from_slice(String::from("hello world").as_bytes());
        c_tx.send(msg).unwrap()
    });

    // Start an event loop.
    let mut events = Events::with_capacity(1024);
    loop {
        // Poll Mio for events, blocking until we get an event.
        poll.poll(&mut events, None).unwrap();
        // Process each event.
        for event in events.iter() {
            debug!("ready {:?}", event);
            if event.readiness().is_readable() {
                match event.token() {
                    SERVER => echo.server.accept(&poll).unwrap(),
                    CLIENT => echo.client.readable(&poll).unwrap(),
                    i => echo.server.conn_readable(&poll, i).unwrap()
                }
            }
            
            if event.readiness().is_writable() {
                match event.token() {
                    SERVER => panic!("received writable for token 10_000_000"),
                    CLIENT => echo.client.writable(&poll).unwrap(),
                    _ => echo.server.conn_writable(&poll, event.token()).unwrap()
                };
            }
        }
    }
}