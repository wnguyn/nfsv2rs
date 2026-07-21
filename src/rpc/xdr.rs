use std::fmt;

pub const RPC_VERSION: u32 = 2;

// WRAP THE HECKIN PRIMATIVES according to RFC 1014
pub struct XdrEncoder {
    buf: Vec<u8>,
}

fn pad(len: usize) -> usize {
    (4 - (len % 4)) % 4
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
