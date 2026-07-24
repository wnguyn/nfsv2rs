use super::msg::OpaqueAuth;
#[derive(Debug)]
pub enum DispatchResult {
    Success(Vec<u8>),
    ProgUnavail,
    ProgMismatch { low: u32, high: u32 },
    ProcUnavail,
    GarbageArgs,
}

pub trait RpcProgram: Send + Sync {
    fn program(&self) -> u32;

    fn version_range(&self) -> (u32, u32);

    fn dispatch(
        &self,
        vers: u32,
        proc: u32,
        cred: &OpaqueAuth,
        verf: &OpaqueAuth,
        args: &[u8],
    ) -> DispatchResult;
}

pub const NFS_PROGRAM: u32 = 100003;
pub const NFS_VERSION: u32 = 2;

// rfc 1094 2.3.1
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum stat {
    NFS_OK=0,
    NFSERR_PERM=1,
    NFSERR_NOENT=2,
    NFSERR_IO=5,
    NFSERR_NXIO=6,
    NFSERR_ACCES=13,
    NFSERR_EXIST=17,
    NFSERR_NODEV=19,
    NFSERR_NOTDIR=20,
    NFSERR_ISDIR=21,
    NFSERR_FBIG=27,
    NFSERR_NOSPC=28,
    NFSERR_ROFS=30,
    NFSERR_NAMETOOLONG = 63,
    NFSERR_NOTEMPTY= 66,
    NFSERR_DQDOT=69,
    NFSERR_STALE=70,
    NFSERR_WFLUSH=99
}

// rfc 1094 2.3.2
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ftype {
    NFNON = 0,
    NFREG = 1,
    NFDIR = 2,
    NFBLK = 3,
    NFCHR = 4,
    NFLNK = 5
}

// this is RFC 1094 2.2 Server Procedures
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum NfsProc {
    Null = 0,
    GetAttr = 1,
    SetAttr = 2,
    Root = 3,
    Lookup = 4,
    ReadLink = 5,
    Read = 6,
    WriteCache = 7,
    Write = 8,
    Create = 9,
    Remove = 10,
    Rename = 11,
    Link = 12,
    SymLink = 13,
    MkDir = 14,
    RmDir = 15,
    ReadDir = 16,
    StatFs = 17,
}
// Return types from ADT
impl NfsProc {
    pub fn from_u32(v: u32) -> Option<Self> {
        match v {
            0 => Some(Self::Null),
            1 => Some(Self::GetAttr),
            2 => Some(Self::SetAttr),
            3 => Some(Self::Root),
            4 => Some(Self::Lookup),
            5 => Some(Self::ReadLink),
            6 => Some(Self::Read),
            7 => Some(Self::WriteCache),
            8 => Some(Self::Write),
            9 => Some(Self::Create),
            10 => Some(Self::Remove),
            11 => Some(Self::Rename),
            12 => Some(Self::Link),
            13 => Some(Self::SymLink),
            14 => Some(Self::MkDir),
            15 => Some(Self::RmDir),
            16 => Some(Self::ReadDir),
            17 => Some(Self::StatFs),
            _ => None,
        }
    }
}
