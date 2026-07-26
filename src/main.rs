mod fs;
mod rpc;
mod transport;

use rpc::program::DispatchResult;
use rpc::xdr::{XdrDecoder, XdrEncoder};
use std::net::UdpSocket;
use std::path::PathBuf;

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

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("NFSv2 server starting, export root: {}", ROOT);

    let socket = UdpSocket::bind("127.0.0.1:2049")?;
    let mount_handler = fs::make_mount_handler(ROOT)?;

    handle(&socket, &mount_handler)
}

fn ver_call(program: &fs::MountHandler, call: &RpcCall<'_>) -> DispatchResult {
    if call.prog != fs::MOUNT_PROGRAM {
        return DispatchResult::ProgUnavail;
    }

    if call.vers != fs::MOUNT_VERSION {
        return DispatchResult::ProgMismatch {
            low: fs::MOUNT_VERSION,
            high: fs::MOUNT_VERSION,
        };
    }

    program.dispatch(call.vers, call.proc_, &call.cred, &call.verf, call.args)
}

fn encode_reply(xid: u32, result: DispatchResult) -> Vec<u8> {
    let mut encoder = XdrEncoder::new();

    encoder.put_u32(xid);
    encoder.put_u32(1);
    encoder.put_u32(0);
    encoder.put_u32(0);
    encoder.put_u32(0);

    match result {
        DispatchResult::Success(payload) => {
            encoder.put_u32(0);
            encoder.put_raw(&payload);
        }
        DispatchResult::ProgUnavail => encoder.put_u32(1),
        DispatchResult::ProgMismatch { low, high } => {
            encoder.put_u32(2);
            encoder.put_u32(low);
            encoder.put_u32(high);
        }
        DispatchResult::ProcUnavail => encoder.put_u32(3),
        DispatchResult::GarbageArgs => encoder.put_u32(4),
    }

    encoder.into_bytes()
}

fn handle(socket: &UdpSocket, program: &fs::MountHandler) -> anyhow::Result<()> {
    let mut buffer = [0u8; 65_536];

    loop {
        let (length, peer) = socket.recv_from(&mut buffer)?;
        let call = match RpcCall::new(&buffer[..length]) {
            Ok(call) => call,
            Err(error) => {
                eprintln!("I need to change this to be a much more elegant if let");
                continue;
            }
        };

        let result = ver_call(program, &call);
        let reply = encode_reply(call.xid, result);
        socket.send_to(&reply, peer)?;
    }
}
