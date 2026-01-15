use crate::error::DBError;
use rusqlite::{params, Connection, ToSql};

pub fn create_table_if_not_exists(
    conn: &Connection,
    name: &str,
    columns: &[(&str, &str)],
) -> Result<(), DBError> {
    let col_defs = columns
        .iter()
        .map(|(name, type_)| format!("{} {}", name, type_))
        .collect::<Vec<_>>()
        .join(", ");

    let sql = format!("CREATE TABLE IF NOT EXISTS {} ({})", name, col_defs);

    conn.execute(&sql, []).map_err(DBError::Sqlite)?;
    Ok(())
}

pub fn create_fts_table(
    conn: &Connection,
    name: &str,
    columns: &[&str],
    content_table: &str,
    content_rowid: &str,
) -> Result<(), DBError> {
    let col_list = columns.join(", ");

    let sql = format!(
        "CREATE VIRTUAL TABLE IF NOT EXISTS {} USING fts5({}, content='{}', content_rowid='{}')",
        name, col_list, content_table, content_rowid
    );

    conn.execute(&sql, []).map_err(DBError::Sqlite)?;
    Ok(())
}

pub fn create_index_if_not_exists(
    conn: &Connection,
    name: &str,
    table: &str,
    columns: &[&str],
    unique: bool,
) -> Result<(), DBError> {
    let col_list = columns.join(", ");
    let unique_str = if unique { "UNIQUE " } else { "" };

    let sql = format!(
        "CREATE {} INDEX IF NOT EXISTS {} ON {}({})",
        unique_str, name, table, col_list
    );

    conn.execute(&sql, []).map_err(DBError::Sqlite)?;
    Ok(())
}

pub fn create_trigger_if_not_exists(
    conn: &Connection,
    name: &str,
    timing: &str,
    event: &str,
    table: &str,
    sql: &str,
) -> Result<(), DBError> {
    let trigger_sql = format!(
        "CREATE TRIGGER IF NOT EXISTS {} {} {} ON {} BEGIN {} END",
        name, timing, event, table, sql
    );

    conn.execute(&trigger_sql, []).map_err(DBError::Sqlite)?;
    Ok(())
}

pub fn drop_table_if_exists(conn: &Connection, name: &str) -> Result<(), DBError> {
    let sql = format!("DROP TABLE IF EXISTS {}", name);
    conn.execute(&sql, []).map_err(DBError::Sqlite)?;
    Ok(())
}

pub fn drop_trigger_if_exists(conn: &Connection, name: &str) -> Result<(), DBError> {
    let sql = format!("DROP TRIGGER IF EXISTS {}", name);
    conn.execute(&sql, []).map_err(DBError::Sqlite)?;
    Ok(())
}

pub fn attach_database(conn: &Connection, name: &str, path: &str) -> Result<(), DBError> {
    conn.execute("ATTACH DATABASE ? AS ?", params![path, name])
        .map_err(DBError::Sqlite)?;
    Ok(())
}

pub fn detach_database(conn: &Connection, name: &str) -> Result<(), DBError> {
    conn.execute("DETACH DATABASE ?", params![name])
        .map_err(DBError::Sqlite)?;
    Ok(())
}

pub fn insert_into<T: ToSql>(
    conn: &Connection,
    table: &str,
    columns: &[&str],
    values: &[&T],
) -> Result<i64, DBError> {
    let col_list = columns.join(", ");
    let placeholders: String = (1..=values.len())
        .map(|_| "?")
        .collect::<Vec<_>>()
        .join(", ");

    let sql = format!(
        "INSERT INTO {} ({}) VALUES ({})",
        table, col_list, placeholders
    );

    conn.execute(&sql, rusqlite::params_from_iter(values.iter()))
        .map_err(DBError::Sqlite)?;

    Ok(conn.last_insert_rowid())
}

pub fn insert_or_replace_into<T: ToSql>(
    conn: &Connection,
    table: &str,
    columns: &[&str],
    values: &[&T],
) -> Result<i64, DBError> {
    let col_list = columns.join(", ");
    let placeholders: String = (1..=values.len())
        .map(|_| "?")
        .collect::<Vec<_>>()
        .join(", ");

    let sql = format!(
        "INSERT OR REPLACE INTO {} ({}) VALUES ({})",
        table, col_list, placeholders
    );

    conn.execute(&sql, rusqlite::params_from_iter(values.iter()))
        .map_err(DBError::Sqlite)?;

    Ok(conn.last_insert_rowid())
}

pub fn update_where<T: ToSql>(
    conn: &Connection,
    table: &str,
    set_parts: &[(&str, &T)],
    where_clause: &str,
    where_params: &[&T],
) -> Result<(), DBError> {
    let set_list = set_parts
        .iter()
        .map(|(col, _)| format!("{} = ?", col))
        .collect::<Vec<_>>()
        .join(", ");

    let mut set_values: Vec<&T> = set_parts.iter().map(|(_, val)| *val).collect();
    set_values.extend(where_params);

    let sql = format!("UPDATE {} SET {} WHERE {}", table, set_list, where_clause);

    conn.execute(&sql, rusqlite::params_from_iter(set_values.iter()))
        .map_err(DBError::Sqlite)?;

    Ok(())
}

pub fn select_where<'a, T: ToSql>(
    conn: &'a Connection,
    table: &'a str,
    columns: &'a [&'a str],
    where_clause: Option<&'a str>,
    order_by: Option<&'a str>,
    limit: Option<u32>,
) -> Result<rusqlite::Statement<'a>, DBError> {
    let col_list = columns.join(", ");

    let mut sql = format!("SELECT {} FROM {}", col_list, table);

    if let Some(where_) = where_clause {
        sql = format!("{} WHERE {}", sql, where_);
    }

    if let Some(order) = order_by {
        sql = format!("{} ORDER BY {}", sql, order);
    }

    if let Some(limit_) = limit {
        sql = format!("{} LIMIT {}", sql, limit_);
    }

    conn.prepare(&sql).map_err(DBError::Sqlite)
}

pub fn delete_where<T: ToSql>(
    conn: &Connection,
    table: &str,
    where_clause: &str,
    where_params: &[&T],
) -> Result<usize, DBError> {
    let sql = format!("DELETE FROM {} WHERE {}", table, where_clause);

    let affected = conn
        .execute(&sql, rusqlite::params_from_iter(where_params.iter()))
        .map_err(DBError::Sqlite)?;

    Ok(affected)
}

pub fn create_table_as_select(
    conn: &Connection,
    new_table: &str,
    old_table: &str,
) -> Result<(), DBError> {
    let sql = format!("CREATE TABLE {} AS SELECT * FROM {}", new_table, old_table);
    conn.execute(&sql, []).map_err(DBError::Sqlite)?;
    Ok(())
}

pub fn pragma_set(conn: &Connection, key: &str, value: &str) -> Result<(), DBError> {
    conn.pragma_update(None, key, value)
        .map_err(DBError::Sqlite)?;
    Ok(())
}
