mod fs;
mod rpc;
mod transport;

use std::net::UdpSocket;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Mutex;

/*
    Ancient Mesopotanian Bell Labs (it isnt actually bell labs)
    Documentation
    https://datatracker.ietf.org/doc/html/rfc1014 //xdr
    https://datatracker.ietf.org/doc/html/rfc1057 //rpc
    https://datatracker.ietf.org/doc/html/rfc1094 //nfs
*/

const ROOT: &'static str = "/home/will/";

// janky; use ENV variables ater
pub struct Config {
    raw_url: String,
    conf: PathBuf,
}

impl Config {
    pub fn new(args: &str) -> Self {
        let path = PathBuf::from(args);
        Self {
            raw_url: format!("{args}"),
            conf: path,
        }
    }
}

impl Clone for Config {
    fn clone(&self) -> Self {
        Self {
            raw_url: self.raw_url.clone(),
            conf: self.conf.clone(),
        }
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("NFSv2 server starting, export root: {}", ROOT);
    let cfg_ptr = Rc::new(Config::new(ROOT));

    let mut buf: [u8; 65536] = [0; 65536];

    // single threaded so just unwrap like a LOSER!!!!
    let listener = Rc::new(Mutex::new(UdpSocket::bind("127.0.0.1:2049")?));
    handle(cfg_ptr.clone(), &listener, buf);
    eprintln!("socket crashed buddy");
    Ok(())
}

fn handle(
    cfg: Rc<Config>,
    socket: &UdpSocket,
    mut buf: [u8; 65536],
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let (amt, src) = socket.recv_from(&mut buf)?;
        transport::process()
    }
    Ok(())
}
