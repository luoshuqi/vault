use rusqlite::Connection;

static VERSION_0: &str = "create table vault
(
    id integer primary key autoincrement,
    key blob not null,
    value blob not null
);
create unique index vault_key_uindex on vault (key);

create table conf
(
    key text not null primary key,
    value text
);";

static VERSIONS: &[&str] = &[VERSION_0];

pub fn setup(conn: &mut Connection) -> crate::Result<()> {
    let mut version = get_version(conn)?;
    if version >= VERSIONS.len() {
        return Ok(());
    }

    let tx = conn.transaction().map_err(err!())?;
    while version < VERSIONS.len() {
        tx.execute_batch(VERSIONS[version]).map_err(err!())?;
        version += 1;
    }

    let sql = if version == 1 {
        "INSERT INTO conf (key, value) VALUES ('version', ?)"
    } else {
        "UPDATE conf SET value=? WHERE key='version'"
    };
    tx.execute(sql, [version]).map_err(err!())?;
    tx.commit().map_err(err!())
}

fn get_version(conn: &Connection) -> crate::Result<usize> {
    let sql = "SELECT COUNT(0) FROM sqlite_master WHERE type='table' AND name='vault'";
    let count: usize = conn.query_row(sql, [], |row| row.get(0)).map_err(err!())?;
    if count == 0 {
        Ok(0)
    } else {
        let sql = "SELECT value FROM conf WHERE key='version'";
        let version: String = conn.query_row(sql, [], |row| row.get(0)).map_err(err!())?;
        version.parse().map_err(err!())
    }
}
