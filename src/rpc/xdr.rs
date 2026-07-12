use std::fmt;

pub const RPC_VERSION: u32 = 2;

#[derive(Debug)]
pub enum XdrError {
    Underflow,
    InvalidBool(u32),
    InvalidUtf8(std::string::FromUtf8Error),
    UnsupportedRpcVersion(u32),
    UnexpectedMsgType,
    BadDiscriminant(u32),
}

impl fmt::Display for XdrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            XdrError::Underflow => write!(f, "buffer underflow"),
            XdrError::InvalidBool(v) => write!(f, "invalid bool value: {v}"),
            XdrError::InvalidUtf8(e) => write!(f, "invalid UTF-8: {e}"),
            XdrError::UnsupportedRpcVersion(v) => write!(f, "unsupported RPC version: {v}"),
            XdrError::UnexpectedMsgType => write!(f, "expected CALL message type"),
            XdrError::BadDiscriminant(v) => write!(f, "bad discriminant: {v}"),
        }
    }
}

impl std::error::Error for XdrError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            XdrError::InvalidUtf8(e) => Some(e),
            _ => None,
        }
    }
}

fn pad(len: usize) -> usize {
    (4 - (len % 4)) % 4
}

pub fn get_u32(buf: &mut &[u8]) -> Result<u32, XdrError> {
    if buf.len() < 4 {
        return Err(XdrError::Underflow);
    }
    let v = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
    *buf = &buf[4..];
    Ok(v)
}

pub fn get_i32(buf: &mut &[u8]) -> Result<i32, XdrError> {
    if buf.len() < 4 {
        return Err(XdrError::Underflow);
    }
    let v = i32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
    *buf = &buf[4..];
    Ok(v)
}

pub fn get_u64(buf: &mut &[u8]) -> Result<u64, XdrError> {
    if buf.len() < 8 {
        return Err(XdrError::Underflow);
    }
    let v = u64::from_be_bytes([
        buf[0], buf[1], buf[2], buf[3],
        buf[4], buf[5], buf[6], buf[7],
    ]);
    *buf = &buf[8..];
    Ok(v)
}

pub fn get_bool(buf: &mut &[u8]) -> Result<bool, XdrError> {
    match get_u32(buf)? {
        0 => Ok(false),
        1 => Ok(true),
        v => Err(XdrError::InvalidBool(v)),
    }
}

pub fn get_opaque_fixed<const N: usize>(buf: &mut &[u8]) -> Result<[u8; N], XdrError> {
    if buf.len() < N {
        return Err(XdrError::Underflow);
    }
    let mut arr = [0u8; N];
    arr.copy_from_slice(&buf[..N]);
    *buf = &buf[N..];
    let p = pad(N);
    if p > 0 {
        if buf.len() < p {
            return Err(XdrError::Underflow);
        }
        *buf = &buf[p..];
    }
    Ok(arr)
}

pub fn get_opaque_var(buf: &mut &[u8]) -> Result<Vec<u8>, XdrError> {
    let len = get_u32(buf)? as usize;
    if buf.len() < len {
        return Err(XdrError::Underflow);
    }
    let v = buf[..len].to_vec();
    *buf = &buf[len..];
    let p = pad(len);
    if p > 0 {
        if buf.len() < p {
            return Err(XdrError::Underflow);
        }
        *buf = &buf[p..];
    }
    Ok(v)
}

pub fn get_string(buf: &mut &[u8]) -> Result<String, XdrError> {
    let bytes = get_opaque_var(buf)?;
    String::from_utf8(bytes).map_err(XdrError::InvalidUtf8)
}

pub fn get_u32_array(buf: &mut &[u8]) -> Result<Vec<u32>, XdrError> {
    let len = get_u32(buf)? as usize;
    let mut out = Vec::with_capacity(len);
    for _ in 0..len {
        out.push(get_u32(buf)?);
    }
    Ok(out)
}

pub struct XdrEncoder {
    buf: Vec<u8>,
}

impl XdrEncoder {
    pub fn new() -> Self {
        Self { buf: Vec::new() }
    }

    pub fn put_u32(&mut self, v: u32) {
        self.buf.extend_from_slice(&v.to_be_bytes());
    }

    pub fn put_i32(&mut self, v: i32) {
        self.buf.extend_from_slice(&v.to_be_bytes());
    }

    pub fn put_u64(&mut self, v: u64) {
        self.buf.extend_from_slice(&v.to_be_bytes());
    }

    pub fn put_bool(&mut self, v: bool) {
        self.put_u32(v as u32);
    }

    pub fn put_opaque_fixed(&mut self, v: &[u8]) {
        self.buf.extend_from_slice(v);
        let p = pad(v.len());
        if p > 0 {
            self.buf.resize(self.buf.len() + p, 0);
        }
    }

    pub fn put_opaque_var(&mut self, v: &[u8]) {
        self.put_u32(v.len() as u32);
        self.buf.extend_from_slice(v);
        let p = pad(v.len());
        if p > 0 {
            self.buf.resize(self.buf.len() + p, 0);
        }
    }

    pub fn put_string(&mut self, v: &str) {
        self.put_opaque_var(v.as_bytes());
    }

    pub fn put_u32_array(&mut self, v: &[u32]) {
        self.put_u32(v.len() as u32);
        for &item in v {
            self.put_u32(item);
        }
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.buf
    }
}

impl Default for XdrEncoder {
    fn default() -> Self {
        Self::new()
    }
}
