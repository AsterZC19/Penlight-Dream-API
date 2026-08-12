use aes::cipher::{BlockDecryptMut, KeyIvInit};
use aes::Aes128;
use cbc::cipher::block_padding::NoPadding;
use cbc::Decryptor;
use thiserror::Error;

type Aes128CbcDec = Decryptor<Aes128>;

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("payload length {0} is not a positive multiple of the AES block size (16)")]
    InvalidPayload(usize),
    #[error("AES-128-CBC decryption failed")]
    DecryptFailed,
}

/// Decrypts a Garupa API response body with AES-128-CBC and no padding.
///
/// The key and IV are the 16-byte values configured per server in
/// `GARUPA_ENCRYPTION_KEYS` and `GARUPA_ENCRYPTION_IVS`. Garupa payloads are
/// always a multiple of the block size, so no padding step is needed.
pub fn decrypt_aes_128_cbc(key: &[u8], iv: &[u8], payload: &[u8]) -> Result<Vec<u8>, CryptoError> {
    #[allow(clippy::manual_is_multiple_of)] // modulo form keeps pre-1.87 toolchains working
    if payload.is_empty() || payload.len() % 16 != 0 {
        return Err(CryptoError::InvalidPayload(payload.len()));
    }

    let mut buf = payload.to_vec();
    let decipher = Aes128CbcDec::new(key.into(), iv.into());
    let out = decipher
        .decrypt_padded_mut::<NoPadding>(&mut buf)
        .map_err(|_| CryptoError::DecryptFailed)?;
    Ok(out.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use aes::cipher::{BlockEncryptMut, KeyIvInit};
    use cbc::Encryptor;

    type Aes128CbcEnc = Encryptor<Aes128>;

    #[test]
    fn round_trip_aes_128_cbc() {
        let key = b"0123456789abcdef";
        let iv = b"fedcba9876543210";
        let plain: &[u8] = b"hello world!!!!!";

        let mut buf = plain.to_vec();
        let _ = Aes128CbcEnc::new(key.into(), iv.into())
            .encrypt_padded_mut::<NoPadding>(&mut buf, plain.len())
            .unwrap();

        let decrypted = decrypt_aes_128_cbc(key, iv, &buf).unwrap();
        assert_eq!(&decrypted, plain);
    }

    #[test]
    fn rejects_non_block_aligned_payload() {
        let key = b"0123456789abcdef";
        let iv = b"fedcba9876543210";
        assert!(decrypt_aes_128_cbc(key, iv, b"short").is_err());
        assert!(decrypt_aes_128_cbc(key, iv, b"").is_err());
    }
}
