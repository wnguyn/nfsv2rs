// actual udp network implementation

use std::io::Cursor;
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::net::UdpSocket;

use crate::rpc::msg::{
    self, garbage_args, proc_unavail, prog_unavail, read_record, success, write_record, ReplyBody,
    RpcCall,
};
use crate::rpc::program::{DispatchResult, RpcProgram};
use crate::rpc::xdr::XdrEncoder;

use crate::Config;
use std::rc::Rc;
use std::sync::Mutex;

fn reply(socket: &UdpSocket, dst: SocketAddr, msg: Vec<u8>) {
    let mut framed = vec![];
    if let Err(e) = write_record(&mut framed, &msg) {
        tracing::error!("write_record failed: {e}");
        return;
    }
    if let Err(e) = socket.send_to(&framed, dst).await {
        tracing::debug!("send_to {dst} failed: {e}");
    }
}
