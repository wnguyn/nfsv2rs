mod fs;
mod rpc;
mod transport;

use parking_lot::Mutex;
use std::net::UdpSocket;
use std::path::PathBuf;
use std::rc::Rc;

use rpc::xdr::XdrDecoder;

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

const RPC_VERSION: u32 = 2;
const CALL: u32 = 0;
const MAX_AUTH_LEN: u32 = 400;

#[derive(Debug)]
pub enum MsgError {
    Xdr,
    NotCall(u32),
    RpcMismatch,
    AuthTooLong(u32),
}

// rfc 1057 8.2
#[derive(Debug, Clone, Copy)]
pub struct OpaqueAuth<'a> {
    pub flavor: u32,
    pub body: &'a [u8],
}

#[derive(Debug)]
pub struct RpcCall<'a> {
    pub xid: u32,
    pub prog: u32,
    pub vers: u32,
    pub proc_: u32,
    pub cred: OpaqueAuth<'a>,
    pub verf: OpaqueAuth<'a>,
    pub args: &'a [u8],
}

impl<'a> RpcCall<'a> {
    pub fn new(buf: &'a [u8]) -> Result<Self, MsgError> {
        let mut d = XdrDecoder::new(buf);

        let xid = d.read_u32().map_err(|_| MsgError::Xdr)?;

        let mtype = d.read_u32().map_err(|_| MsgError::Xdr)?;
        if mtype != CALL {
            return Err(MsgError::NotCall(mtype));
        }

        if d.read_u32().map_err(|_| MsgError::Xdr)? != RPC_VERSION {
            return Err(MsgError::RpcMismatch);
        }

        let prog = d.read_u32().map_err(|_| MsgError::Xdr)?;
        let vers = d.read_u32().map_err(|_| MsgError::Xdr)?;
        let proc_ = d.read_u32().map_err(|_| MsgError::Xdr)?;

        let cred_flavor = d.read_u32().map_err(|_| MsgError::Xdr)?;
        let cred_len = d.read_u32().map_err(|_| MsgError::Xdr)?;
        if cred_len > MAX_AUTH_LEN {
            return Err(MsgError::AuthTooLong(cred_len));
        }
        let cred = OpaqueAuth {
            flavor: cred_flavor,
            body: d
                .read_opaque_fixed(cred_len as usize)
                .map_err(|_| MsgError::Xdr)?,
        };

        let verf_flavor = d.read_u32().map_err(|_| MsgError::Xdr)?;
        let verf_len = d.read_u32().map_err(|_| MsgError::Xdr)?;
        if verf_len > MAX_AUTH_LEN {
            return Err(MsgError::AuthTooLong(verf_len));
        }
        let verf = OpaqueAuth {
            flavor: verf_flavor,
            body: d
                .read_opaque_fixed(verf_len as usize)
                .map_err(|_| MsgError::Xdr)?,
        };

        Ok(Self {
            xid,
            prog,
            vers,
            proc_,
            cred,
            verf,
            args: d.remaining(),
        })
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("NFSv2 server starting, export root: {}", ROOT);
    let cfg_ptr = Rc::new(Config::new(ROOT));

    let buf: [u8; 65536] = [0; 65536];

    let listener = Rc::new(Mutex::new(UdpSocket::bind("127.0.0.1:2049")?));
    handle(cfg_ptr.clone(), listener.clone(), buf)?;
    eprintln!("ya stuff crashed man");
    Ok(())
}

pub struct Message {}

fn handle(
    _cfg: Rc<Config>,
    socket: Rc<Mutex<UdpSocket>>,
    mut buf: [u8; 65536],
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let (amt, src) = socket.lock().recv_from(&mut buf)?;
        let datagram = &buf[..amt]; // slice to datagram; past amt is stale zeros

        let call = match RpcCall::new(datagram) {
            Ok(c) => c,
            Err(e) => {
                tracing::debug!("dropping garbage from {src}: {e:?}");
                continue; // rfc 1057: silent drop is legal
            }
        };
    }
    Ok(())
}
