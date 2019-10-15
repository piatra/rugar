use std::net::{TcpListener, TcpStream};
use std::io;
use std::io::Write;
use std::io::BufRead;



fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:3012")?;
    let mut count: usize = 1;

    // accept connections and process them serially
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                stream.write_all(format!("{}", count).as_bytes())?;
                count += 1;

                loop {
                    let bytes_read = stream.read(&mut buf)?;
                    if bytes_read == 0 { return Ok(()); }
                    // stream.write(&buf[..bytes_read])?;
                }
            }
            Err(e) => { /* connection failed */ }
        }
    }
    Ok(())
}