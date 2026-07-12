use std::error::Error;

pub trait Decode: Sized {
    fn decode(buf: &[u8]) -> Result<Self, Box<dyn Error>>;
}

pub trait Encode: Sized {
    fn encode(&self, buf: &mut Vec<u8>);
}

fn pad_len(len: usize) -> usize {
    (4 - (len % 4)) % 4
}

impl Encode for u32 {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.to_be_bytes());
    }
}

impl Decode for u32 {
    fn decode(buf: &[u8]) -> Result<Self, Box<dyn Error>> {
        if buf.len() < 4 {
            return Err("not enough bytes".into());
        }
        let bytes = [buf[0], buf[1], buf[2], buf[3]];
        Ok(u32::from_be_bytes(bytes))
    }
}

impl Encode for i32 {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.to_be_bytes());
    }
}

impl Decode for i32 {
    fn decode(buf: &[u8]) -> Result<Self, Box<dyn Error>> {
        if buf.len() < 4 {
            return Err("not enough bytes".into());
        }
        let bytes = [buf[0], buf[1], buf[2], buf[3]];
        Ok(i32::from_be_bytes(bytes))
    }
}

impl Encode for u64 {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.to_be_bytes());
    }
}

impl Decode for u64 {
    fn decode(buf: &[u8]) -> Result<Self, Box<dyn Error>> {
        if buf.len() < 8 {
            return Err("not enough bytes".into());
        }
        let bytes = [
            buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7],
        ];
        Ok(u64::from_be_bytes(bytes))
    }
}

impl Encode for i64 {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.to_be_bytes());
    }
}

impl Decode for i64 {
    fn decode(buf: &[u8]) -> Result<Self, Box<dyn Error>> {
        if buf.len() < 8 {
            return Err("not enough bytes".into());
        }
        let bytes = [
            buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7],
        ];
        Ok(i64::from_be_bytes(bytes))
    }
}

impl Encode for f32 {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.to_be_bytes());
    }
}

impl Decode for f32 {
    fn decode(buf: &[u8]) -> Result<Self, Box<dyn Error>> {
        if buf.len() < 4 {
            return Err("not enough bytes".into());
        }
        let bytes = [buf[0], buf[1], buf[2], buf[3]];
        Ok(f32::from_be_bytes(bytes))
    }
}

impl Encode for f64 {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.to_be_bytes());
    }
}

impl Decode for f64 {
    fn decode(buf: &[u8]) -> Result<Self, Box<dyn Error>> {
        if buf.len() < 8 {
            return Err("not enough bytes".into());
        }
        let bytes = [
            buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7],
        ];
        Ok(f64::from_be_bytes(bytes))
    }
}

impl Encode for bool {
    fn encode(&self, buf: &mut Vec<u8>) {
        (*self as u32).encode(buf);
    }
}

impl Decode for bool {
    fn decode(buf: &[u8]) -> Result<Self, Box<dyn Error>> {
        match u32::decode(buf)? {
            0 => Ok(false),
            1 => Ok(true),
            v => Err(format!("invalid bool value: {}", v).into()),
        }
    }
}

impl Encode for () {
    fn encode(&self, _buf: &mut Vec<u8>) {}
}

impl Decode for () {
    fn decode(_buf: &[u8]) -> Result<Self, Box<dyn Error>> {
        Ok(())
    }
}

impl Encode for Vec<u8> {
    fn encode(&self, buf: &mut Vec<u8>) {
        (self.len() as u32).encode(buf);
        buf.extend_from_slice(self);
        for _ in 0..pad_len(self.len()) {
            buf.push(0);
        }
    }
}

impl Decode for Vec<u8> {
    fn decode(buf: &[u8]) -> Result<Self, Box<dyn Error>> {
        let len = u32::decode(buf)? as usize;
        let pad = pad_len(len);
        if buf.len() < 4 + len + pad {
            return Err("not enough bytes".into());
        }
        Ok(buf[4..4 + len].to_vec())
    }
}

impl Encode for String {
    fn encode(&self, buf: &mut Vec<u8>) {
        let bytes = self.as_bytes();
        (bytes.len() as u32).encode(buf);
        buf.extend_from_slice(bytes);
        for _ in 0..pad_len(bytes.len()) {
            buf.push(0);
        }
    }
}

impl Decode for String {
    fn decode(buf: &[u8]) -> Result<Self, Box<dyn Error>> {
        String::from_utf8(Vec::decode(buf)?).map_err(|e| e.into())
    }
}



impl Encode for [u8; 32] {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(self);
        for _ in 0..pad_len(32) {
            buf.push(0);
        }
    }
}

impl Decode for [u8; 32] {
    fn decode(buf: &[u8]) -> Result<Self, Box<dyn Error>> {
        if buf.len() < 32 {
            return Err("not enough bytes".into());
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&buf[..32]);
        Ok(arr)
    }
}
