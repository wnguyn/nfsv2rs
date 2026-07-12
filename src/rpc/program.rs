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
