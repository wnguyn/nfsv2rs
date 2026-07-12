use std::error::Error;

pub trait Decode: Sized {
    fn decode(buf: &[u8]) -> Result<Self, Box<dyn Error>>;
}

pub trait Encode: Sized {
    fn encode(&self, buf: &mut Vec<u8>);
}

fn take<const N: usize>(buf: &[u8]) -> Result<[u8; N], Box<dyn Error>> {
    if buf.len() < N {
        return Err("not enough bytes".into());
    }
    let mut bytes = [0u8; N];
    bytes.copy_from_slice(&buf[..N]);
    Ok(bytes)
}

fn pad_len(len: usize) -> usize {
    (4 - (len % 4)) % 4
}

macro_rules! impl_fixed {
    ($ty:ty, $n:expr) => {
        impl Encode for $ty {
            fn encode(&self, buf: &mut Vec<u8>) {
                buf.extend_from_slice(&self.to_be_bytes());
            }
        }

        impl Decode for $ty {
            fn decode(buf: &[u8]) -> Result<Self, Box<dyn Error>> {
                Ok(<$ty>::from_be_bytes(take::<$n>(buf)?))
            }
        }
    };
}

impl_fixed!(u32, 4);
impl_fixed!(i32, 4);
impl_fixed!(u64, 8);
impl_fixed!(i64, 8);
impl_fixed!(f32, 4);
impl_fixed!(f64, 8);

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

impl<const N: usize> Encode for [u8; N] {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self[..]);
        buf.resize(buf.len() + pad_len(N), 0);
    }
}

impl<const N: usize> Decode for [u8; N] {
    fn decode(buf: &[u8]) -> Result<Self, Box<dyn Error>> {
        let pad = pad_len(N);
        if buf.len() < N + pad {
            return Err("not enough bytes".into());
        }
        let mut arr = [0u8; N];
        arr.copy_from_slice(&buf[..N]);
        Ok(arr)
    }
}

impl Encode for Vec<u8> {
    fn encode(&self, buf: &mut Vec<u8>) {
        let len = self.len() as u32;
        len.encode(buf);
        buf.extend_from_slice(self);
        buf.resize(buf.len() + pad_len(self.len()), 0);
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
        buf.resize(buf.len() + pad_len(bytes.len()), 0);
    }
}

impl Decode for String {
    fn decode(buf: &[u8]) -> Result<Self, Box<dyn Error>> {
        String::from_utf8(Vec::decode(buf)?).map_err(|e| e.into())
    }
}

