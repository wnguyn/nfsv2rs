use std::path::PathBuf;
use crate::rpc::xdr::XdrEncoder;


// TODO: use inode numbers later but for a poc it's fine i think
// MAX HECKIN 32 BYTES FOR FILES!!!
const FH_SIZE: usize = 32;
const FH: u32 = 0x4E465332;








mod PathHandle {
    use std::path::PathBuf;
    use crate::rpc::xdr::XdrEncoder;

    pub fn path_to_handle(path: &PathBuf) -> [u8; 32] {
        use std::os::unix::ffi::OsStrExt;
        let mut handle = [0u8; 32];
        let bytes = path.as_os_str().as_bytes();
        let len = bytes.len().min(32);
        handle[..len].copy_from_slice(&bytes[..len]);
        handle
    }
    
    pub fn handle_to_path(handle: [u8; 32]) -> PathBuf {
        use std::os::unix::ffi::OsStrExt;
        let end = handle.iter().position(|&b| b == 0).unwrap_or(32);
        PathBuf::from(std::ffi::OsStr::from_bytes(&handle[..end]))
    }











}


