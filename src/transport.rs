// actual udp network implementation


use std::io::Cursor;
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::net::UdpSocket;

use crate::rpc::msg::{self, garbage_args, proc_unavail, prog_unavail, read_record, success, write_record, RpcCall, ReplyBody};
use crate::rpc::program::{DispatchResult, RpcProgram};
use crate::rpc::xdr::XdrEncoder;

use crate::Config;

pub async fn serve(cfg_ptr: Box<Config>, nfs: Arc<dyn RpcProgram>, mount: Arc<dyn RpcProgram>, bind_addr: &str) -> anyhow::Result<()> {
    let socket = Arc::new(UdpSocket::bind(bind_addr).await?);
    let mut buf = vec![0u8; 65536];

    tracing::info!("listening on {}", bind_addr);

    loop {
        buf.resize(65536, 0);
        
        let (n, src) = match socket.recv_from(&mut buf).await {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("recv_from failed: {e}");
                continue;
            }
        };

        let payload = buf[..n].to_vec();
        let nfs = Arc::clone(&nfs);
        let mount = Arc::clone(&mount);
        let socket = Arc::clone(&socket);
        let cfg = cfg_ptr.clone();

        tokio::spawn(async move {
            process(payload, src, cfg, nfs, mount, socket).await;
        });
    }
}


// tldr; strips framing -> decode -> route by program number -> check version
async fn process(
    payload: Vec<u8>,
    src: SocketAddr,
    cfg: Box<Config>,
    nfs: Arc<dyn RpcProgram>,
    mount: Arc<dyn RpcProgram>,
    socket: Arc<UdpSocket>,
) {
    let xid_hint = peek_xid(&payload);
    let record = match read_record(&mut Cursor::new(&payload)) {
        Ok(r) => r,
        Err(e) => {
            let id = xid_hint.map_or("?".into(), |x| x.to_string());
            tracing::debug!("[xid={id}] bad record framing from {src}: {e}");
            return;
        }
    };

    let mut remaining = &record[..];
    let call = match RpcCall::decode(&mut remaining) {
        Ok(c) => c,
        Err(e) => {
            let xid = xid_hint.unwrap_or(0);
            tracing::debug!("[xid={xid}] decode failed from {src}: {e}");
            reply(&socket, src, garbage_args(xid).into_bytes()).await;
            return;
        }
    };
    let args = remaining.to_vec();

    let xid = call.xid;
    tracing::debug!(
        "[xid={xid}] prog={} vers={} proc={} from {src}",
        call.body.prog, call.body.vers, call.body.proc
    );

    let handler: Arc<dyn RpcProgram> = match call.body.prog {
        crate::rpc::program::NFS_PROGRAM => Arc::clone(&nfs),
        crate::fs::MOUNT_PROGRAM => Arc::clone(&mount),
        _ => {
            reply(&socket, src, prog_unavail(xid).into_bytes()).await;
            return;
        }
    };

    let (lo, hi) = handler.version_range();
    if call.body.vers < lo || call.body.vers > hi {
        let mut e = XdrEncoder::new();
        msg::encode_header(
            xid,
            &ReplyBody::Accepted {
                verf: msg::Auth::None,
                stat: msg::AcceptStat::ProgMismatch,
                mismatch_info: Some((lo, hi)),
            },
            &mut e,
        );
        reply(&socket, src, e.into_bytes()).await;
        return;
    }

    let dispatch_fut = tokio::task::spawn_blocking(move || {
        handler.dispatch(
            call.body.vers,
            call.body.proc,
            &call.body.cred,
            &call.body.verf,
            &args,
        )
    });

    let result = match dispatch_fut.await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("[xid={xid}] spawn_blocking join error: {e}");
            return;
        }
    };

    let reply_bytes = match result {
        DispatchResult::Success(data) => {
            let mut e = success(xid);
            e.put_raw(&data);
            e.into_bytes()
        }
        DispatchResult::ProgUnavail => prog_unavail(xid).into_bytes(),
        DispatchResult::ProgMismatch { low, high } => {
            let mut e = XdrEncoder::new();
            msg::encode_header(
                xid,
                &ReplyBody::Accepted {
                    verf: msg::Auth::None,
                    stat: msg::AcceptStat::ProgMismatch,
                    mismatch_info: Some((low, high)),
                },
                &mut e,
            );
            e.into_bytes()
        }
        DispatchResult::ProcUnavail => proc_unavail(xid).into_bytes(),
        DispatchResult::GarbageArgs => garbage_args(xid).into_bytes(),
    };

    reply(&socket, src, reply_bytes).await;
}


fn peek_xid(payload: &[u8]) -> Option<u32> {
    let xid_bytes = payload.get(4..8)?;
    Some(u32::from_be_bytes(xid_bytes.try_into().unwrap()))
}

async fn reply(socket: &UdpSocket, dst: SocketAddr, msg: Vec<u8>) {
    let mut framed = vec![];
    if let Err(e) = write_record(&mut framed, &msg) {
        tracing::error!("write_record failed: {e}");
        return;
    }
    if let Err(e) = socket.send_to(&framed, dst).await {
        tracing::debug!("send_to {dst} failed: {e}");
    }
}
