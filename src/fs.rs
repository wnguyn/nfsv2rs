use std::path::PathBuf;
use std::sync::Arc;

use crate::rpc::msg::Auth;
use crate::rpc::program::{DispatchResult, RpcProgram};
use crate::rpc::xdr::{get_string, XdrEncoder};

pub const MOUNT_PROGRAM: u32 = 100005;
pub const MOUNT_VERSION: u32 = 1;

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

#[derive(Debug, Clone)]
pub struct MountHandler {
    export_root: PathBuf,
}

impl MountHandler {
    pub fn new(export_root: impl Into<PathBuf>) -> anyhow::Result<Self> {
        let export_root = export_root.into();
        if !export_root.is_dir() {
            anyhow::bail!("export root is not a directory: {}", export_root.display());
        }
        Ok(Self {
            export_root: export_root.canonicalize()?,
        })
    }
}

impl RpcProgram for MountHandler {
    fn program(&self) -> u32 {
        MOUNT_PROGRAM
    }

    fn version_range(&self) -> (u32, u32) {
        (MOUNT_VERSION, MOUNT_VERSION)
    }

    fn dispatch(
        &self,
        vers: u32,
        proc: u32,
        _cred: &Auth,
        _verf: &Auth,
        args: &[u8],
    ) -> DispatchResult {
        if vers != MOUNT_VERSION {
            return DispatchResult::ProgMismatch {
                low: MOUNT_VERSION,
                high: MOUNT_VERSION,
            };
        }

        // HECKING RFC 1094 2.3.1
        match proc {
            0 => DispatchResult::Success(Vec::new()),
            1 => {
                let mut buf = args;
                let path_str = match get_string(&mut buf) {
                    Ok(s) => s,
                    Err(_) => return DispatchResult::GarbageArgs,
                };

                let resolved = match self
                    .export_root
                    .join(path_str.trim_start_matches('/'))
                    .canonicalize()
                {
                    Ok(p) => p,
                    Err(_) => {
                        let mut enc = XdrEncoder::new();
                        enc.put_u32(1); // FHS_ERROR
                        return DispatchResult::Success(enc.into_bytes());
                    }
                };

                if !resolved.starts_with(&self.export_root) {
                    let mut enc = XdrEncoder::new();
                    enc.put_u32(1); // FHS_ERROR
                    return DispatchResult::Success(enc.into_bytes());
                }

                let handle = path_to_handle(&resolved);
                let mut enc = XdrEncoder::new();
                enc.put_u32(0); // FHS_OK
                enc.put_opaque_fixed(&handle);
                DispatchResult::Success(enc.into_bytes())
            }
            _ => DispatchResult::Success(Vec::new()),
        }
    }
}

pub fn make_mount_handler(export_root: impl Into<PathBuf>) -> anyhow::Result<Arc<dyn RpcProgram>> {
    Ok(Arc::new(MountHandler::new(export_root)?))
}
