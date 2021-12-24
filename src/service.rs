use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::convert::Infallible;
use std::fs::{read, remove_file, OpenOptions};

use log::error;
use openssl::rand::rand_bytes;
use rand::distributions::{Distribution, Uniform};
use rand::prelude::SliceRandom;
use rusqlite::{params, ToSql};
use serde::{Deserialize, Serialize};
use serde_json::{from_slice, json};
use ws_jsonrpc::response::Error as RpcError;
use ws_jsonrpc::{method, rpc, Method};

use crate::crypto::{key_encrypt, password_decrypt, password_encrypt};
use crate::server::{close_any_addr, db, listen_any_addr, query_network_port};
use crate::service::Error::WrongPassword;

#[derive(Debug)]
enum Error {
    // 密码错误
    WrongPassword,

    // json 解析失败
    DeserializeFailed,

    // 其他错误
    Any(crate::Error),
}

impl From<crate::Error> for Error {
    fn from(err: crate::error::Error) -> Self {
        Self::Any(err)
    }
}

impl From<Error> for RpcError {
    fn from(err: Error) -> Self {
        match err {
            Error::Any(err) => err.into(),
            _ => {
                let data = json!({ "kind": format!("{:?}", err) });
                RpcError::server_error(None, "Server Error", Some(data))
            }
        }
    }
}

// 主密码是否设置
#[rpc]
fn is_master_password_set() -> crate::Result<bool> {
    const SQL: &str = "SELECT COUNT(0) FROM conf WHERE key='key'";
    let count: u32 = db()
        .conn()
        .map_err(err!())?
        .query_row(SQL, [], |row| row.get(0))
        .map_err(err!())?;
    Ok(count == 1)
}

// 设置主密码
#[rpc]
fn set_master_password(master_password: String) -> crate::Result<()> {
    let mut key = [0u8; 32];
    rand_bytes(&mut key).map_err(err!())?;

    let key = password_encrypt(master_password, key).map_err(err!())?;
    let key = base64::encode(key);

    const SQL: &str = "INSERT INTO conf (key, value) VALUES ('key', ?)";
    db().conn()
        .map_err(err!())?
        .execute(SQL, [key])
        .map_err(err!())?;
    Ok(())
}

// 验证主密码
#[rpc]
fn verify_master_password(master_password: String) -> Result<bool, Error> {
    match decrypt_master_key(master_password) {
        Ok(_) => Ok(true),
        Err(WrongPassword) => Ok(false),
        Err(e) => Err(e),
    }
}

#[derive(Serialize)]
struct Item {
    id: u64,
    name: String,
}

// 获取所有密码
#[rpc]
fn list_password(master_password: String) -> Result<Vec<Item>, Error> {
    let key = decrypt_master_key(master_password)?;
    let db = db();
    let mut stmt = db
        .conn()
        .map_err(err!())?
        .prepare("SELECT id, key FROM vault ORDER BY id")
        .map_err(err!())?;
    let mut rows = stmt.query([]).map_err(err!())?;
    let mut list = Vec::new();
    loop {
        match rows.next().map_err(err!())? {
            Some(row) => {
                let name: Vec<u8> = row.get(1).map_err(err!())?;
                let name = key_decrypt(&key, name)?;
                list.push(Item {
                    id: row.get(0).map_err(err!())?,
                    name: String::from_utf8(name).map_err(err!())?,
                })
            }
            None => break,
        }
    }
    Ok(list)
}

#[derive(Serialize)]
struct Password {
    name: String,
    password: String,
}

// 获取单个密码
#[rpc]
fn get_password(master_password: String, id: u64) -> Result<Password, Error> {
    let key = decrypt_master_key(master_password)?;
    const SQL: &str = "SELECT key, value FROM vault WHERE id=?";
    let (name, password): (Vec<u8>, Vec<u8>) = db()
        .conn()
        .map_err(err!())?
        .query_row(SQL, [id], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(err!())?;

    let name = key_decrypt(&key, name)?;
    let password = key_decrypt(key, password)?;
    Ok(Password {
        name: String::from_utf8(name).map_err(err!())?,
        password: String::from_utf8(password).map_err(err!())?,
    })
}

// 生成密码
#[rpc]
fn make_password(option: PasswordOption) -> Result<String, Infallible> {
    let chars = option.chars();
    if chars.is_empty() || option.len == 0 || option.len > 8192 {
        return Ok(String::new());
    }

    let mut password = Vec::with_capacity(option.len as _);
    let len = option.len / chars.len();
    let mut remainder = option.len % chars.len();
    let mut rng = rand::thread_rng();
    for c in chars {
        let uniform = Uniform::new(0, c.len());
        for _ in 0..len {
            password.push(c[uniform.sample(&mut rng)] as _)
        }
        if remainder > 0 {
            remainder -= 1;
            password.push(c[uniform.sample(&mut rng)] as _)
        }
    }
    password.shuffle(&mut rng);
    unsafe { Ok(String::from_utf8_unchecked(password)) }
}

// 添加密码
#[rpc]
fn add_password(master_password: String, name: String, password: String) -> Result<(), Error> {
    let key = decrypt_master_key(master_password)?;
    let name = key_encrypt(&key, name).map_err(err!())?;
    let password = key_encrypt(key, password).map_err(err!())?;

    const SQL: &str = "INSERT INTO vault (key, value) VALUES (?, ?)";
    db().conn()
        .map_err(err!())?
        .execute(SQL, params![name, password])
        .map_err(err!())?;
    Ok(())
}

// 修改密码
#[rpc]
fn change_password(master_password: String, new_password: String) -> Result<(), Error> {
    let key = decrypt_master_key(master_password)?;
    let new_key = password_encrypt(new_password, key).map_err(err!())?;
    const SQL: &str = "UPDATE conf SET value=? WHERE key='key'";
    db().conn()
        .map_err(err!())?
        .execute(SQL, [base64::encode(new_key)])
        .map_err(err!())?;
    Ok(())
}

// 更新密码
#[rpc]
fn update_password(
    master_password: String,
    id: u64,
    name: String,
    password: String,
) -> Result<(), Error> {
    let key = decrypt_master_key(master_password)?;
    let name = key_encrypt(&key, name).map_err(err!())?;
    let password = key_encrypt(key, password).map_err(err!())?;
    const SQL: &str = "UPDATE vault SET key=?, value=? WHERE id=?";
    db().conn()
        .map_err(err!())?
        .execute(SQL, params![name, password, id])
        .map_err(err!())?;
    Ok(())
}

// 删除密码
#[rpc]
fn delete_password(master_password: String, id: u64) -> Result<(), Error> {
    decrypt_master_key(master_password)?;
    db().conn()
        .map_err(err!())?
        .execute("DELETE FROM vault WHERE id=?", [id])
        .map_err(err!())?;
    Ok(())
}

#[derive(Serialize)]
struct Count {
    ignore: usize,
    insert: usize,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Source {
    // 文件
    File(String),
    // 数据
    Data(Vec<(String, String)>),
}

// 导入密码
#[rpc]
fn import_password(
    master_password: String,
    decrypt_password: Option<String>,
    source: Source,
) -> Result<Count, Error> {
    let key = decrypt_master_key(&master_password)?;
    let mut data = match source {
        Source::File(path) => {
            let data = read(&path).map_err(err!());
            let _ = remove_file(path);
            match from_slice(&data?) {
                Ok(data) => data,
                Err(err) => {
                    error!("{:?}", err);
                    return Err(Error::DeserializeFailed);
                }
            }
        }
        Source::Data(data) => data,
    };

    let mut count = Count {
        ignore: 0,
        insert: 0,
    };
    if data.is_empty() {
        return Ok(count);
    }

    let decrypt_password = decrypt_password.unwrap_or(master_password);
    let decrypt_key = base64::decode(data.pop().unwrap().0).map_err(err!())?;
    let decrypt_key = match password_decrypt(decrypt_password, decrypt_key).map_err(err!())? {
        Some(key) => key,
        None => return Err(WrongPassword),
    };

    let entries = get_all_password_as_map(&key)?;
    let mut insert = Vec::new();
    for (name, password) in data {
        let name = key_decrypt(&decrypt_key, &base64::decode(&name).map_err(err!())?)?;
        let password = key_decrypt(&decrypt_key, &base64::decode(&password).map_err(err!())?)?;
        match entries.get(&name) {
            Some(v) if v.contains(&password) => count.ignore += 1,
            _ => {
                count.insert += 1;
                insert.push(key_encrypt(&key, name).map_err(err!())?);
                insert.push(key_encrypt(&key, password).map_err(err!())?);
            }
        }
    }

    if !insert.is_empty() {
        let mut values = Vec::with_capacity(insert.len());
        for v in &insert {
            values.push(v.to_sql().map_err(err!())?);
        }

        let placeholder: Vec<_> = (0..insert.len()).step_by(2).map(|_| "(?, ?)").collect();
        let sql = &format!(
            "INSERT INTO vault (key, value) VALUES {}",
            placeholder.join(", ")
        );
        let values: Vec<&dyn ToSql> = values.iter().map(|v| -> &dyn ToSql { v }).collect();
        db().conn()
            .map_err(err!())?
            .execute(sql, values.as_slice())
            .map_err(err!())?;
    }

    Ok(count)
}

// 所有密码
fn get_all_password_as_map(key: &[u8]) -> Result<HashMap<Vec<u8>, HashSet<Vec<u8>>>, Error> {
    let mut map: HashMap<Vec<u8>, HashSet<Vec<u8>>> = HashMap::new();
    for (name, password) in get_all_password()? {
        let name = key_decrypt(key, &name)?;
        let password = key_decrypt(key, &password)?;
        match map.entry(name) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().insert(password);
            }
            Entry::Vacant(entry) => {
                let mut s = HashSet::new();
                s.insert(password);
                entry.insert(s);
            }
        }
    }
    Ok(map)
}

// 所有密码
fn get_all_password() -> crate::Result<Vec<(Vec<u8>, Vec<u8>)>> {
    let mut list = Vec::new();
    let db = db();
    let mut stmt = db
        .conn()
        .map_err(err!())?
        .prepare("SELECT key, value FROM vault ORDER BY id")
        .map_err(err!())?;
    let mut rows = stmt.query([]).map_err(err!())?;
    loop {
        match rows.next().map_err(err!())? {
            Some(row) => {
                let name: Vec<u8> = row.get(0).map_err(err!())?;
                let value: Vec<u8> = row.get(1).map_err(err!())?;
                list.push((name, value));
            }
            None => break,
        }
    }
    Ok(list)
}

// 导出密码, 如果 file 不为 None，导出到 file，返回 None，否则返回数据
#[rpc]
fn export_password(
    master_password: String,
    file: Option<String>,
) -> Result<Option<Vec<(String, String)>>, Error> {
    let key = decrypt_master_key(&master_password)?;

    let mut list: Vec<_> = get_all_password()?
        .into_iter()
        .map(|(name, value)| (base64::encode(name), base64::encode(value)))
        .collect();

    if !list.is_empty() {
        let encrypt_key = password_encrypt(master_password, key).map_err(err!())?;
        list.push((base64::encode(encrypt_key), String::new()));
    }

    match file {
        Some(file) => {
            let file = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(file)
                .map_err(err!())?;
            serde_json::to_writer(file, &list).map_err(err!())?;
            Ok(None)
        }
        None => Ok(Some(list)),
    }
}

#[rpc]
async fn get_network_port(master_password: String) -> Result<Option<u16>, Error> {
    decrypt_master_key(master_password.as_bytes())?;
    Ok(query_network_port().await?)
}

#[rpc]
async fn enable_network_access(master_password: String) -> Result<u16, Error> {
    decrypt_master_key(master_password.as_bytes())?;
    Ok(listen_any_addr().await?)
}

#[rpc]
fn disable_network_access(master_password: String) -> Result<(), Error> {
    decrypt_master_key(master_password.as_bytes())?;
    Ok(close_any_addr()?)
}

// 解密密码加密使用的 key
fn decrypt_master_key(password: impl AsRef<[u8]>) -> Result<Vec<u8>, Error> {
    const SQL: &str = "SELECT value FROM conf WHERE key='key'";
    let key: String = db()
        .conn()
        .map_err(err!())?
        .query_row(SQL, [], |row| row.get(0))
        .map_err(err!())?;
    let key = base64::decode(key).map_err(err!())?;
    match password_decrypt(password.as_ref(), &key).map_err(err!())? {
        Some(key) => Ok(key),
        None => Err(WrongPassword),
    }
}

fn key_decrypt(key: impl AsRef<[u8]>, data: impl AsRef<[u8]>) -> Result<Vec<u8>, Error> {
    match crate::crypto::key_decrypt(key.as_ref(), data.as_ref()).map_err(err!())? {
        Some(data) => Ok(data),
        None => Err(WrongPassword),
    }
}

#[derive(Deserialize)]
struct PasswordOption {
    len: usize,
    uppercase: bool,
    lowercase: bool,
    digit: bool,
    special: bool,
}

impl PasswordOption {
    fn chars(&self) -> Vec<&'static [u8]> {
        static UPPERCASE: &'static [u8] = &[
            b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H', b'I', b'J', b'K', b'L', b'M', b'N',
            b'O', b'P', b'Q', b'R', b'S', b'T', b'U', b'V', b'W', b'X', b'Y', b'Z',
        ];
        static LOWERCASE: &'static [u8] = &[
            b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h', b'i', b'j', b'k', b'l', b'm', b'n',
            b'o', b'p', b'q', b'r', b's', b't', b'u', b'v', b'w', b'x', b'y', b'z',
        ];
        static DIGIT: &'static [u8] = &[b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9'];
        static SPECIAL: &'static [u8] = &[
            b'!', b'"', b'#', b'$', b'%', b'&', b'\'', b'(', b')', b'*', b'+', b',', b'-', b'.',
            b'/', b':', b';', b'<', b'=', b'>', b'?', b'@', b'[', b'\\', b']', b'^', b'_', b'`',
            b'{', b'|', b'}', b'~',
        ];

        let mut chars = Vec::with_capacity(4);
        if self.uppercase {
            chars.push(UPPERCASE)
        }
        if self.lowercase {
            chars.push(LOWERCASE)
        }
        if self.digit {
            chars.push(DIGIT)
        }
        if self.special {
            chars.push(SPECIAL)
        }
        chars
    }
}

pub fn methods() -> Vec<(&'static str, Method)> {
    vec![
        method!(is_master_password_set),
        method!(set_master_password),
        method!(verify_master_password),
        method!(make_password),
        method!(add_password),
        method!(list_password),
        method!(get_password),
        method!(delete_password),
        method!(export_password),
        method!(import_password),
        method!(update_password),
        method!(change_password),
        method!(get_network_port),
        method!(enable_network_access),
        method!(disable_network_access),
    ]
}
