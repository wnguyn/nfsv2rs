use super::xdr::{get_opaque_var, get_string, get_u32, get_u32_array, RPC_VERSION, XdrEncoder, XdrError};
use std::io::{Read, Write};

/* uses the numbers on enums specified in 
*  RFC 5531 and is obviously below nfs */

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MsgType {
    Call = 0,
    Reply = 1,
}

impl MsgType {
    pub fn from_u32(v: u32) -> Result<Self, XdrError> {
        match v {
            0 => Ok(MsgType::Call),
            1 => Ok(MsgType::Reply),
            _ => Err(XdrError::BadDiscriminant(v)),
        }
    }
}

#[derive(Debug, Clone)]
pub enum OpaqueAuth {
    None,
    Unix(AuthUnix),
    Short(Vec<u8>),
}

#[derive(Debug, Clone)]
pub struct AuthUnix {
    pub stamp: u32,
    pub machine_name: String,
    pub uid: u32,
    pub gid: u32,
    pub gids: Vec<u32>,
}

impl OpaqueAuth {
    pub fn decode(buf: &mut &[u8]) -> Result<Self, XdrError> {
        let flavor = get_u32(buf)?;
        let body = get_opaque_var(buf)?;

        match flavor {
            0 => Ok(OpaqueAuth::None),
            1 => {
                let mut inner = body.as_slice();
                let stamp = get_u32(&mut inner)?;
                let machine_name = get_string(&mut inner)?;
                let uid = get_u32(&mut inner)?;
                let gid = get_u32(&mut inner)?;
                let gids = get_u32_array(&mut inner)?;
                Ok(OpaqueAuth::Unix(AuthUnix { stamp, machine_name, uid, gid, gids }))
            }
            _ => Ok(OpaqueAuth::Short(body)),
        }
    }

    pub fn encode(&self, e: &mut XdrEncoder) {
        match self {
            OpaqueAuth::None => {
                e.put_u32(0);
                e.put_opaque_var(&[]);
            }
            OpaqueAuth::Unix(au) => {
                let mut bd = XdrEncoder::new();
                bd.put_u32(au.stamp);
                bd.put_string(&au.machine_name);
                bd.put_u32(au.uid);
                bd.put_u32(au.gid);
                bd.put_u32_array(&au.gids);
                e.put_u32(1);
                e.put_opaque_var(&bd.into_bytes());
            }
            OpaqueAuth::Short(body) => {
                e.put_u32(2);
                e.put_opaque_var(body);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct CallBody {
    pub rpcvers: u32,
    pub prog: u32,
    pub vers: u32,
    pub proc: u32,
    pub cred: OpaqueAuth,
    pub verf: OpaqueAuth,
}

impl CallBody {
    pub fn decode(buf: &mut &[u8]) -> Result<Self, XdrError> {
        let rpcvers = get_u32(buf)?;
        if rpcvers != RPC_VERSION {
            return Err(XdrError::UnsupportedRpcVersion(rpcvers));
        }
        let prog = get_u32(buf)?;
        let vers = get_u32(buf)?;
        let proc = get_u32(buf)?;
        let cred = OpaqueAuth::decode(buf)?;
        let verf = OpaqueAuth::decode(buf)?;
        Ok(CallBody { rpcvers, prog, vers, proc, cred, verf })
    }
}

#[derive(Clone, Copy)]
pub enum AcceptStat {
    Success = 0,
    ProgUnavail = 1,
    ProgMismatch = 2,
    ProcUnavail = 3,
    GarbageArgs = 4,
    SystemErr = 5,
}

#[derive(Clone)]
pub enum ReplyBody {
    Accepted {
        verf: OpaqueAuth,
        stat: AcceptStat,
        mismatch_info: Option<(u32, u32)>,
    },
    Rejected(RejectedReply),
}

#[derive(Clone)]
pub enum RejectedReply {
    RpcMismatch { low: u32, high: u32 },
    AuthError(u32),
}

impl ReplyBody {
    pub fn encode(&self, e: &mut XdrEncoder) {
        match self {
            ReplyBody::Accepted { verf, stat, mismatch_info } => {
                e.put_u32(0);
                verf.encode(e);
                e.put_u32(*stat as u32);
                if let AcceptStat::ProgMismatch = stat {
                    let (lo, hi) = mismatch_info.unwrap_or((0, 0));
                    e.put_u32(lo);
                    e.put_u32(hi);
                }
            }
            ReplyBody::Rejected(r) => {
                e.put_u32(1);
                match r {
                    RejectedReply::RpcMismatch { low, high } => {
                        e.put_u32(0);
                        e.put_u32(*low);
                        e.put_u32(*high);
                    }
                    RejectedReply::AuthError(stat) => {
                        e.put_u32(1);
                        e.put_u32(*stat);
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct RpcCall {
    pub xid: u32,
    pub body: CallBody,
}

impl RpcCall {
    pub fn decode(buf: &mut &[u8]) -> Result<Self, XdrError> {
        let xid = get_u32(buf)?;
        let mtype = MsgType::from_u32(get_u32(buf)?)?;
        if mtype != MsgType::Call {
            return Err(XdrError::UnexpectedMsgType);
        }
        let body = CallBody::decode(buf)?;
        Ok(RpcCall { xid, body })
    }
}


pub fn encode_header(xid: u32, body: &ReplyBody, e: &mut XdrEncoder) {
    e.put_u32(xid);
    e.put_u32(MsgType::Reply as u32);
    body.encode(e);
}

pub fn success(xid: u32) -> XdrEncoder {
    let mut e = XdrEncoder::new();
    encode_header(
        xid,
        &ReplyBody::Accepted {
            verf: OpaqueAuth::None,
            stat: AcceptStat::Success,
            mismatch_info: None,
        },
        &mut e,
    );
    e
}

pub fn prog_unavail(xid: u32) -> XdrEncoder {
    let mut e = XdrEncoder::new();
    encode_header(
        xid,
        &ReplyBody::Accepted {
            verf: OpaqueAuth::None,
            stat: AcceptStat::ProgUnavail,
            mismatch_info: None,
        },
        &mut e,
    );
    e
}

pub fn proc_unavail(xid: u32) -> XdrEncoder {
    let mut e = XdrEncoder::new();
    encode_header(
        xid,
        &ReplyBody::Accepted {
            verf: OpaqueAuth::None,
            stat: AcceptStat::ProcUnavail,
            mismatch_info: None,
        },
        &mut e,
    );
    e
}

pub fn garbage_args(xid: u32) -> XdrEncoder {
    let mut e = XdrEncoder::new();
    encode_header(
        xid,
        &ReplyBody::Accepted {
            verf: OpaqueAuth::None,
            stat: AcceptStat::GarbageArgs,
            mismatch_info: None,
        },
        &mut e,
    );
    e
}

pub fn read_record<R: Read>(r: &mut R) -> Result<Vec<u8>, std::io::Error> {
    let mut out = Vec::new();
    loop {
        let mut hdr = [0u8; 4];
        r.read_exact(&mut hdr)?;
        let n = u32::from_be_bytes(hdr);
        let last = (n & 0x8000_0000) != 0;
        let len = (n & 0x7fff_ffff) as usize;

        let mut frag = vec![0u8; len];
        r.read_exact(&mut frag)?;
        out.extend_from_slice(&frag);

        if last {
            break;
        }
    }
    Ok(out)
}

pub fn write_record<W: Write>(w: &mut W, msg: &[u8]) -> Result<(), std::io::Error> {
    let n = (msg.len() as u32) | 0x8000_0000;
    w.write_all(&n.to_be_bytes())?;
    w.write_all(msg)?;
    Ok(())
}
