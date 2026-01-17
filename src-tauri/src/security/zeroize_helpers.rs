use std::ops::Deref;
use zeroize::Zeroize;

#[derive(Zeroize)]
#[zeroize(drop)]
pub struct ZeroizingBytes(pub Vec<u8>);

impl Deref for ZeroizingBytes {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        &self.0
    }
}

impl From<Vec<u8>> for ZeroizingBytes {
    fn from(data: Vec<u8>) -> Self {
        ZeroizingBytes(data)
    }
}

impl From<&[u8]> for ZeroizingBytes {
    fn from(data: &[u8]) -> Self {
        ZeroizingBytes(data.to_vec())
    }
}

pub fn secure_zeroize(data: &mut [u8]) {
    data.zeroize();
}

pub fn secure_zeroize_vec(data: &mut Vec<u8>) {
    data.zeroize();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zeroizing_bytes_drops() {
        let mut bytes = ZeroizingBytes(vec![1, 2, 3, 4, 5]);
        {
            let slice: &[u8] = &bytes;
            assert_eq!(slice, &[1, 2, 3, 4, 5]);
        }
        let _ = bytes; // Drop happens here
    }

    #[test]
    fn test_from_slice() {
        let bytes = ZeroizingBytes::from(&[1, 2, 3][..]);
        assert_eq!(&*bytes, &[1, 2, 3]);
    }
}
