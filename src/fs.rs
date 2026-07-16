use std::path::PathBuf;
use crate::rpc::xdr::XdrEncoder;


// TODO: use inode numbers later but for a poc it's fine i think
const FH_SIZE: usize = 32;
const FH: u32 = 0x4E465332;

pub struct NfsPath {
    path: PathBuf,
    bytes: XdrEncoder,
}

impl NfsPath {

    pub fn new(stream



}





