use openssl::error::ErrorStack;
use openssl::rand::rand_bytes;
use openssl::symm::{decrypt_aead, encrypt_aead, Cipher};
use sha2::digest::Output;
use sha2::{Digest, Sha256};

// AES-256-GCM iv 长度为 12
const IV_LEN: usize = 12;

// AES-256-GCM tag 长度为 16
const TAG_LEN: usize = 16;

// 密码加密
pub fn password_encrypt(
    password: impl AsRef<[u8]>,
    data: impl AsRef<[u8]>,
) -> Result<Vec<u8>, ErrorStack> {
    key_encrypt(calc_key(password), data)
}

// key 加密
pub fn key_encrypt(key: impl AsRef<[u8]>, data: impl AsRef<[u8]>) -> Result<Vec<u8>, ErrorStack> {
    let data = data.as_ref();
    if data.is_empty() {
        return Ok(Vec::new());
    }
    let key = key.as_ref();

    let mut iv = [0u8; IV_LEN];
    rand_bytes(&mut iv)?;
    let mut tag = [0u8; TAG_LEN];

    let mut output = encrypt_aead(Cipher::aes_256_gcm(), key, Some(&iv), &[], data, &mut tag)?;
    output.reserve(IV_LEN + TAG_LEN);
    output.extend_from_slice(&iv);
    output.extend_from_slice(&tag);
    Ok(output)
}

// 密码解密
pub fn password_decrypt(
    password: impl AsRef<[u8]>,
    data: impl AsRef<[u8]>,
) -> Result<Option<Vec<u8>>, ErrorStack> {
    key_decrypt(calc_key(password), data)
}

// key 解密
pub fn key_decrypt(
    key: impl AsRef<[u8]>,
    data: impl AsRef<[u8]>,
) -> Result<Option<Vec<u8>>, ErrorStack> {
    let data = data.as_ref();
    if data.len() <= IV_LEN + TAG_LEN {
        // 长度不够，原样返回
        return Ok(Some(data.to_vec()));
    }

    let data_len = data.len() - TAG_LEN - IV_LEN;
    let tag = &data[data_len + IV_LEN..];
    let iv = &data[data_len..data_len + IV_LEN];
    let cipher = Cipher::aes_256_gcm();
    match decrypt_aead(cipher, key.as_ref(), Some(iv), &[], &data[..data_len], tag) {
        Ok(data) => Ok(Some(data)),
        Err(err) if err.errors().is_empty() => Ok(None),
        Err(err) => Err(err),
    }
}

// 把密码转为 key
fn calc_key(password: impl AsRef<[u8]>) -> Output<Sha256> {
    const SALT: &[u8] = &[
        230, 220, 184, 57, 90, 105, 50, 133, 76, 108, 175, 186, 142, 138, 95, 16,
    ];

    let mut hasher = Sha256::new();
    hasher.update(password);
    hasher.update(SALT);
    hasher.finalize()
}
