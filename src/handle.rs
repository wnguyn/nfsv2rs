use std::path::PathBuf;
use std::sync::Arc;

use crate::rpc::msg::OpaqueAuth;
use crate::rpc::program::{DispatchResult, NfsProc, NFS_PROGRAM, NFS_VERSION, RpcProgram};

pub struct NfsHandler {
    export_root: PathBuf,
}

impl NfsHandler {
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

impl RpcProgram for NfsHandler {
    fn program(&self) -> u32 {
        NFS_PROGRAM
    }

    fn version_range(&self) -> (u32, u32) {
        (NFS_VERSION, NFS_VERSION)
    }

    fn dispatch(
        &self,
        vers: u32,
        proc: u32,
        _cred: &OpaqueAuth,
        _verf: &OpaqueAuth,
        _args: &[u8],
    ) -> DispatchResult {
        if vers != NFS_VERSION {
            return DispatchResult::ProgMismatch {
                low: NFS_VERSION,
                high: NFS_VERSION,
            };
        }

        match NfsProc::from_u32(proc) {
            Some(NfsProc::Null) => DispatchResult::Success(Vec::new()),
            Some(_) => DispatchResult::ProcUnavail,
            None => DispatchResult::ProcUnavail,
        }
    }
}

pub fn make_handler(export_root: impl Into<PathBuf>) -> anyhow::Result<Arc<dyn RpcProgram>> {
    Ok(Arc::new(NfsHandler::new(export_root)?))
}
