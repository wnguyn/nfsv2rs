use crate::fs::{handle_to_path, path_to_handle};
use crate::rpc::program::{DispatchResult, NFS_VERSION};
use crate::rpc::xdr::{XdrDecoder, XdrEncoder};
use crate::OpaqueAuth;
use std::fs::{self, File, Metadata};
use std::io::{Read, Seek, SeekFrom};
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;

const FILE_HANDLE_SIZE: usize = 32;
const NFS_MAX_READ: u32 = 8192;

#[derive(Debug, Clone)]
pub struct NfsHandler {
    export_root: PathBuf,
}
/* See RFC 1057 to understand what these numbers mean because its literally just
what it is documented in that ancient sun microsystems text */

impl NfsHandler {
    pub fn new(export_root: impl Into<PathBuf>) -> anyhow::Result<Self> {
        let export_root: PathBuf = export_root.into();
        if !export_root.is_dir() {
            anyhow::bail!("export root is not a directory: {}", export_root.display());
        }
        Ok(Self {
            export_root: export_root.canonicalize()?,
        })
    }

    pub fn dispatch(
        &self,
        vers: u32,
        proc: u32,
        _cred: &OpaqueAuth<'_>,
        _verf: &OpaqueAuth<'_>,
        args: &[u8],
    ) -> DispatchResult {
        if vers != NFS_VERSION {
            return DispatchResult::ProgMismatch {
                low: NFS_VERSION,
                high: NFS_VERSION,
            };
        }
        match proc {
            0 => DispatchResult::Success(Vec::new()),
            1 => self.getattr(args),
            4 => self.lookup(args),
            6 => self.read(args),
            16 => self.readdir(args),
            _ => DispatchResult::ProcUnavail,
        }
    }

    fn resolve_handle(&self, bytes: &[u8]) -> Option<PathBuf> {
        let handle: [u8; FILE_HANDLE_SIZE] = bytes.try_into().ok()?;
        let raw_path = handle_to_path(handle);
        if raw_path.as_os_str().is_empty() || !raw_path.is_absolute() {
            return None;
        }
        let path = raw_path.canonicalize().ok()?;
        path.starts_with(&self.export_root).then_some(path)
    }

    fn status_error(error: &std::io::Error) -> u32 {
        use std::io::ErrorKind;
        match error.kind() {
            ErrorKind::NotFound => 2,
            ErrorKind::PermissionDenied => 13,
            ErrorKind::AlreadyExists => 17,
            ErrorKind::InvalidInput => 63,
            _ => 5, // IO error
        }
    }

    fn encode_status(status: u32) -> DispatchResult {
        let mut enc = XdrEncoder::new();
        enc.put_u32(status);
        DispatchResult::Success(enc.into_bytes())
    }

    fn getattr(&self, args: &[u8]) -> DispatchResult {
        let mut d = XdrDecoder::new(args);
        let handle = match d.read_opaque_fixed(FILE_HANDLE_SIZE) {
            Ok(v) => v,
            Err(_) => return DispatchResult::GarbageArgs,
        };
        let path = match self.resolve_handle(handle) {
            Some(p) => p,
            None => return Self::encode_status(70), // STALE REQ!!
        };
        match fs::symlink_metadata(path) {
            Ok(metadata) => {
                let mut enc = XdrEncoder::new();
                enc.put_u32(0);
                put_fattr(&mut enc, &metadata);
                DispatchResult::Success(enc.into_bytes())
            }
            Err(e) => Self::encode_status(Self::status_error(&e)),
        }
    }

    fn lookup(&self, args: &[u8]) -> DispatchResult {
        let mut d = XdrDecoder::new(args);
        let dir_handle = match d.read_opaque_fixed(FILE_HANDLE_SIZE) {
            Ok(v) => v,
            Err(_) => return DispatchResult::GarbageArgs,
        };
        let name = match d.read_string() {
            Ok(v) => v,
            Err(_) => return DispatchResult::GarbageArgs,
        };
        if name.is_empty() || name.contains('/') {
            return Self::encode_status(2);
        }
        let dir = match self.resolve_handle(dir_handle) {
            Some(p) => p,
            None => return Self::encode_status(70),
        };
        let dir_meta = match fs::symlink_metadata(&dir) {
            Ok(m) => m,
            Err(e) => return Self::encode_status(Self::status_error(&e)),
        };
        if !dir_meta.is_dir() {
            return Self::encode_status(20);
        }
        let path = match dir.join(name).canonicalize() {
            Ok(p) if p.starts_with(&self.export_root) => p,
            Ok(_) => return Self::encode_status(13),
            Err(e) => return Self::encode_status(Self::status_error(&e)),
        };
        let metadata = match fs::symlink_metadata(&path) {
            Ok(m) => m,
            Err(e) => return Self::encode_status(Self::status_error(&e)),
        };
        let mut enc = XdrEncoder::new();
        enc.put_u32(0);
        enc.put_opaque_fixed(&path_to_handle(&path));
        put_fattr(&mut enc, &metadata);
        DispatchResult::Success(enc.into_bytes())
    }

    fn read(&self, args: &[u8]) -> DispatchResult {
        let mut d = XdrDecoder::new(args);
        let handle = match d.read_opaque_fixed(FILE_HANDLE_SIZE) {
            Ok(v) => v,
            Err(_) => return DispatchResult::GarbageArgs,
        };
        let offset = match d.read_u32() {
            Ok(v) => v,
            Err(_) => return DispatchResult::GarbageArgs,
        };
        let count = match d.read_u32() {
            Ok(v) => v,
            Err(_) => return DispatchResult::GarbageArgs,
        };
        if d.read_u32().is_err() {
            return DispatchResult::GarbageArgs;
        }
        let path = match self.resolve_handle(handle) {
            Some(p) => p,
            None => return Self::encode_status(70),
        };
        let metadata = match fs::symlink_metadata(&path) {
            Ok(m) => m,
            Err(e) => return Self::encode_status(Self::status_error(&e)),
        };
        if metadata.is_dir() {
            return Self::encode_status(21);
        }
        let read_len = count.min(NFS_MAX_READ) as usize;
        let mut file = match File::open(&path) {
            Ok(f) => f,
            Err(e) => return Self::encode_status(Self::status_error(&e)),
        };
        if file.seek(SeekFrom::Start(offset as u64)).is_err() {
            return Self::encode_status(5);
        }
        let mut data = vec![0u8; read_len];
        let n = match file.read(&mut data) {
            Ok(n) => n,
            Err(e) => return Self::encode_status(Self::status_error(&e)),
        };
        data.truncate(n);
        let eof = offset as u64 + n as u64 >= metadata.len();
        let mut enc = XdrEncoder::new();
        enc.put_u32(0);
        put_fattr(&mut enc, &metadata);
        enc.put_u32(n as u32);
        enc.put_opaque_fixed(&data);
        enc.put_bool(eof);
        DispatchResult::Success(enc.into_bytes())
    }

    fn readdir(&self, args: &[u8]) -> DispatchResult {
        let mut d = XdrDecoder::new(args);
        let handle = match d.read_opaque_fixed(FILE_HANDLE_SIZE) {
            Ok(v) => v,
            Err(_) => return DispatchResult::GarbageArgs,
        };
        let cookie = match d.read_u32() {
            Ok(v) => v,
            Err(_) => return DispatchResult::GarbageArgs,
        };
        let count = match d.read_u32() {
            Ok(v) => v,
            Err(_) => return DispatchResult::GarbageArgs,
        };
        let path = match self.resolve_handle(handle) {
            Some(p) => p,
            None => return Self::encode_status(70),
        };
        match fs::symlink_metadata(&path) {
            Ok(m) if !m.is_dir() => return Self::encode_status(20),
            Err(e) => return Self::encode_status(Self::status_error(&e)),
            _ => {}
        }
        let entries: Vec<_> = match fs::read_dir(&path) {
            Ok(iter) => iter.filter_map(Result::ok).collect(),
            Err(e) => return Self::encode_status(Self::status_error(&e)),
        };
        let start = cookie as usize;
        let mut enc = XdrEncoder::new();
        enc.put_u32(0);
        let mut used = 4usize;
        let mut next = start;
        for (index, entry) in entries.iter().enumerate().skip(start) {
            let name = entry.file_name().to_string_lossy().into_owned();
            let encoded = 4 + 4 + 4 + ((name.len() + 3) & !3) + 4;
            if used + encoded + 4 > count as usize {
                break;
            }
            let fileid = entry.metadata().map(|m| m.ino()).unwrap_or(0) as u32;
            enc.put_bool(true);
            enc.put_u32(fileid);
            enc.put_string(&name);
            enc.put_u32((index + 1) as u32);
            used += encoded;
            next = index + 1;
        }
        let eof = next >= entries.len();
        enc.put_bool(false);
        enc.put_bool(eof);
        DispatchResult::Success(enc.into_bytes())
    }
}
// Dear god
fn put_fattr(enc: &mut XdrEncoder, metadata: &Metadata) {
    let kind = metadata.file_type();
    let ftype = if kind.is_file() {
        1
    } else if kind.is_dir() {
        2
    } else if kind.is_symlink() {
        5
    } else {
        0
    };
    enc.put_u32(ftype);
    enc.put_u32(metadata.mode() & 0o7777);
    enc.put_u32(metadata.nlink() as u32);
    enc.put_u32(metadata.uid());
    enc.put_u32(metadata.gid());
    enc.put_u32(metadata.len().min(u32::MAX as u64) as u32);
    enc.put_u32(metadata.blksize().min(u32::MAX as u64) as u32);
    enc.put_u32(metadata.rdev() as u32);
    enc.put_u32(metadata.blocks().min(u32::MAX as u64) as u32);
    enc.put_u32(metadata.dev() as u32);
    enc.put_u32(metadata.ino() as u32);
    put_time(enc, metadata.atime(), metadata.atime_nsec());
    put_time(enc, metadata.mtime(), metadata.mtime_nsec());
    put_time(enc, metadata.ctime(), metadata.ctime_nsec());
}

fn put_time(enc: &mut XdrEncoder, seconds: i64, nanos: i64) {
    enc.put_u32(seconds.max(0).min(u32::MAX as i64) as u32);
    enc.put_u32((nanos.max(0) / 1000).min(u32::MAX as i64) as u32);
}

pub fn make_nfs_handler(export_root: impl Into<PathBuf>) -> anyhow::Result<NfsHandler> {
    NfsHandler::new(export_root)
}
