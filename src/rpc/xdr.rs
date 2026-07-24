use std::fmt;

pub const RPC_VERSION: u32 = 2;

fn pad(len: usize) -> usize {
    (4 - (len % 4)) % 4
}

pub struct XdrDecoder<'a> {
    buf: &'a [u8],
    pos: usize,
}

#[derive(Debug)]
pub enum XdrError {
    Eof,
    Err,
    // more later
}

impl fmt::Display for XdrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            XdrError::Eof => write!(f, "helsadjfkasdf"),
            XdrError::Err => write!(f, "placeholasdkfjhasdfkjhasdfkjh"),
            // generic error
        }
    }
}

impl<'a> XdrDecoder<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn remaining(&self) -> &'a [u8] {
        &self.buf[self.pos..]
    }

    fn align(&mut self) {
        let p = pad(self.pos);
        self.pos += p;
    }

    fn need(&self, n: usize) -> Result<(), XdrError> {
        if self.pos + n > self.buf.len() {
            Err(XdrError::Eof)
        } else {
            Ok(())
        }
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8], XdrError> {
        self.need(n)?;
        let slice = &self.buf[self.pos..self.pos + n];
        self.pos += n;
        Ok(slice)
    }

    pub fn read_u32(&mut self) -> Result<u32, XdrError> {
        let bytes: [u8; 4] = self.take(4)?.try_into().unwrap();
        Ok(u32::from_be_bytes(bytes))
    }

    pub fn read_i32(&mut self) -> Result<i32, XdrError> {
        let bytes: [u8; 4] = self.take(4)?.try_into().unwrap();
        Ok(i32::from_be_bytes(bytes))
    }

    pub fn read_u64(&mut self) -> Result<u64, XdrError> {
        let bytes: [u8; 8] = self.take(8)?.try_into().unwrap();
        Ok(u64::from_be_bytes(bytes))
    }

    pub fn read_bool(&mut self) -> Result<bool, XdrError> {
        Ok(self.read_u32()? != 0)
    }

    pub fn read_opaque_fixed(&mut self, len: usize) -> Result<&'a [u8], XdrError> {
        let data = self.take(len)?;
        self.align();
        Ok(data)
    }

    pub fn read_opaque_var(&mut self) -> Result<&'a [u8], XdrError> {
        let len = self.read_u32()? as usize;
        self.read_opaque_fixed(len)
    }

    pub fn read_string(&mut self) -> Result<&'a str, XdrError> {
        let len = self.read_u32()? as usize;
        let bytes = self.read_opaque_fixed(len)?;
        std::str::from_utf8(bytes).map_err(|_| XdrError::Err)
    }

    pub fn read_u32_array(&mut self) -> Result<Vec<u32>, XdrError> {
        let len = self.read_u32()? as usize;
        let mut out = Vec::with_capacity(len);
        for _ in 0..len {
            out.push(self.read_u32()?);
        }
        Ok(out)
    }
}
// WRAP THE HECKIN PRIMATIVES according to RFC 1014
// (3-3.16)
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
    pub fn put_raw(&mut self, v: &[u8]) {
        self.buf.extend_from_slice(v);
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
