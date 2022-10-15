#![allow(unused_imports)]
#![allow(deprecated)]
use std::net::{
    TcpListener, TcpStream,
};
use std::io::{self, Read, Write};
use std::{
    thread, 
    time::{Instant, Duration},
};
pub use ports::localhost;

pub fn init_log() {
    env_logger::init_from_env(
        env_logger::Env::new()
        .filter_or("TEST_LOG", "RUST_LOG")
        .write_style_or("TEST_LOG_STYLE", "RUST_LOG_STYLE"),
    );
}

pub fn timed<F>(label: &str, f: F)
where F: Fn() -> ()
{
    let start = Instant::now();
    f();
    let elapsed = start.elapsed();
    println!("  {}: {}", label, elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 / 1_000_000_000.0);
}

pub fn mock_server() {
    let addr = "0.0.0.0:50052";
    log::debug!("[+] server startd at: {}", addr);
    let server = TcpListener::bind(addr).unwrap();
    for stream in server.incoming() {
        // A single stream represents an open connection between the client and the server.
        let stream = stream.unwrap();
        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    // let get = b"GET / HTTP/1.1\r\n";
    // if buffer.starts_with(get) {
    //     for x in 0..10 {
    //         let status_line = "HTTP/1.1 200 OK";
    //         let contents = format!("{} says: hello world!", x);
        
    //         let response = format!(
    //             "{}\r\nContent-Length: {}\r\n\r\n{}",
    //             status_line,
    //             contents.len(),
    //             contents
    //         );
        
    //         stream.write(response.as_bytes()).unwrap();
    //         stream.flush().unwrap();    
    //     }
    // }
    let status_line = "HTTP/1.1 200 OK";
    let contents = format!("hello world!");

    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    );

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();             // flush will wait and prevent the program from continuing until all the bytes are written to the connection; 
                                        // TcpStream contains an internal buffer to minimize calls to the underlying operating system.
}

mod ports {
    use std::net::SocketAddr;
    use std::str::FromStr;
    use std::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT};
    use std::sync::atomic::Ordering::SeqCst;

    // Helper for getting a unique port for the task run
    // TODO: Reuse ports to not spam the system
    static mut NEXT_PORT: AtomicUsize = ATOMIC_USIZE_INIT;
    const FIRST_PORT: usize = 18080;

    fn next_port() -> usize {
        unsafe {
            // If the atomic was never used, set it to the initial port
            NEXT_PORT.compare_and_swap(0, FIRST_PORT, SeqCst);

            // Get and increment the port list
            NEXT_PORT.fetch_add(1, SeqCst)
        }
    }

    pub fn localhost() -> SocketAddr {
        let s = format!("127.0.0.1:{}", next_port());
        FromStr::from_str(&s).unwrap()
    }
}

use bytes::{Buf, MutBuf};
pub trait TryRead {
    // Reads the length of the slice supplied by buf.mut_bytes into the buffer
    // This is not guaranteed to consume an entire datagram or segment.
    // If your protocol is msg based (instead of continuous stream) you should
    // ensure that your buffer is large enough to hold an entire segment (1532 bytes if not jumbo
    // frames)
    fn try_read_buf<B: MutBuf>(&mut self, buf: &mut B) -> io::Result<Option<usize>> 
    where Self : Sized
    {
        let res = self.try_read(unsafe { buf.mut_bytes() });
        if let Ok(Some(cnt)) = res {
            unsafe { buf.advance(cnt) }
        }

        res
    }

    fn try_read(&mut self, buf: &mut [u8]) -> io::Result<Option<usize>>;
}

pub trait TryWrite {
    fn try_write_buf<B: Buf>(&mut self, buf: &mut B) -> io::Result<Option<usize>> 
    where Self : Sized
    {
        let res = self.try_write(buf.bytes());
        if let Ok(Some(cnt)) = res {
            buf.advance(cnt);
        }

        res
    }

    fn try_write(&mut self, buf: &[u8]) -> io::Result<Option<usize>>;
}

impl<T: Read> TryRead for T {
    fn try_read(&mut self, dst: &mut [u8]) -> io::Result<Option<usize>> {
        self.read(dst).map_non_block()
    }
}

impl<T: Write> TryWrite for T {
    fn try_write(&mut self, src: &[u8]) -> io::Result<Option<usize>> {
        self.write(src).map_non_block()
    }
}


/*
 *
 * ===== Helpers =====
 *
 */

/// A helper trait to provide the map_non_block function on Results.
trait MapNonBlock<T> {
    /// Maps a `Result<T>` to a `Result<Option<T>>` by converting
    /// operation-would-block errors into `Ok(None)`.
    fn map_non_block(self) -> io::Result<Option<T>>;
}

impl<T> MapNonBlock<T> for io::Result<T> {
    fn map_non_block(self) -> io::Result<Option<T>> {
        use std::io::ErrorKind::WouldBlock;

        match self {
            Ok(value) => Ok(Some(value)),
            Err(err) => {
                if let WouldBlock = err.kind() {
                    Ok(None)
                } else {
                    Err(err)
                }
            }
        }
    }
}