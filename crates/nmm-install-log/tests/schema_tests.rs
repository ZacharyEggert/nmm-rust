use rusqlite::Connection;

/// Open a fresh in-memory DB with foreign keys enabled and apply the schema.
fn open_db() -> Connection {
    let conn = Connection::open_in_memory().expect("open_in_memory failed");
    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .expect("PRAGMA foreign_keys failed");
    nmm_install_log::schema::apply(&conn).expect("schema::apply failed");
    conn
}

// ─── 1. Schema applies cleanly on a fresh DB ─────────────────────────────

#[test]
fn schema_applies_on_fresh_db() {
    let _conn = open_db();
}

// ─── 2. Idempotency ───────────────────────────────────────────────────────

#[test]
fn schema_apply_is_idempotent() {
    let conn = open_db();
    nmm_install_log::schema::apply(&conn).expect("second apply failed");
}

// ─── 3. All required tables exist ─────────────────────────────────────────

#[test]
fn all_tables_exist() {
    let conn = open_db();
    let tables = [
        "schema_meta",
        "mods",
        "file_owners",
        "ini_edits",
        "gsv_edits",
    ];
    for table in &tables {
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                [table],
                |row| row.get(0),
            )
            .unwrap_or(0);
        assert_eq!(count, 1, "table '{}' must exist", table);
    }
}

// ─── 4. All required indices exist ────────────────────────────────────────

#[test]
fn all_indices_exist() {
    let conn = open_db();
    let indices = [
        "idx_file_owners_by_path",
        "idx_file_owners_by_mod",
        "idx_ini_edits_by_key",
        "idx_ini_edits_by_mod",
        "idx_gsv_edits_by_key",
        "idx_gsv_edits_by_mod",
    ];
    for idx in &indices {
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name=?1",
                [idx],
                |row| row.get(0),
            )
            .unwrap_or(0);
        assert_eq!(count, 1, "index '{}' must exist", idx);
    }
}

// ─── 5. schema_version seed row ───────────────────────────────────────────

#[test]
fn schema_version_row_exists() {
    let conn = open_db();
    let version: i64 = conn
        .query_row(
            "SELECT int_value FROM schema_meta WHERE key = 'schema_version'",
            [],
            |row| row.get(0),
        )
        .expect("schema_version row must exist");
    assert_eq!(version, nmm_install_log::schema::CURRENT_VERSION);
}

// ─── 6. install_order_seq seed row ────────────────────────────────────────

#[test]
fn install_order_seq_initialized() {
    let conn = open_db();
    let seq: i64 = conn
        .query_row(
            "SELECT int_value FROM schema_meta WHERE key = 'install_order_seq'",
            [],
            |row| row.get(0),
        )
        .expect("install_order_seq row must exist");
    assert_eq!(seq, 0);
}

// ─── 7. mods PK uniqueness ────────────────────────────────────────────────

#[test]
fn mods_pk_uniqueness() {
    let conn = open_db();
    conn.execute(
        "INSERT INTO mods (mod_key, archive_path, name) VALUES ('abc123', 'mods/test.7z', 'Test Mod')",
        [],
    )
    .expect("first insert must succeed");

    let result = conn.execute(
        "INSERT INTO mods (mod_key, archive_path, name) VALUES ('abc123', 'mods/other.7z', 'Other Mod')",
        [],
    );
    assert!(result.is_err(), "duplicate mod_key must be rejected");
}

// ─── 8. file_owners composite PK ──────────────────────────────────────────

#[test]
fn file_owners_pk_uniqueness() {
    let conn = open_db();
    conn.execute(
        "INSERT INTO mods (mod_key, archive_path, name) VALUES ('mod1', 'a.7z', 'A')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO file_owners (file_path, mod_key, install_order) VALUES ('Data/test.dds', 'mod1', 1)",
        [],
    )
    .expect("first insert must succeed");

    let result = conn.execute(
        "INSERT INTO file_owners (file_path, mod_key, install_order) VALUES ('Data/test.dds', 'mod1', 2)",
        [],
    );
    assert!(
        result.is_err(),
        "duplicate (file_path, mod_key) must be rejected"
    );
}

// ─── 9. ini_edits composite PK ────────────────────────────────────────────

#[test]
fn ini_edits_pk_uniqueness() {
    let conn = open_db();
    conn.execute(
        "INSERT INTO mods (mod_key, archive_path, name) VALUES ('mod1', 'a.7z', 'A')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO ini_edits (ini_file, section, key, mod_key, value, install_order) \
         VALUES ('skyrim.ini', 'Display', 'fShadowDistance', 'mod1', '1500', 1)",
        [],
    )
    .expect("first insert must succeed");

    let result = conn.execute(
        "INSERT INTO ini_edits (ini_file, section, key, mod_key, value, install_order) \
         VALUES ('skyrim.ini', 'Display', 'fShadowDistance', 'mod1', '2000', 2)",
        [],
    );
    assert!(
        result.is_err(),
        "duplicate (ini_file, section, key, mod_key) must be rejected"
    );
}

// ─── 10. gsv_edits composite PK ───────────────────────────────────────────

#[test]
fn gsv_edits_pk_uniqueness() {
    let conn = open_db();
    conn.execute(
        "INSERT INTO mods (mod_key, archive_path, name) VALUES ('mod1', 'a.7z', 'A')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO gsv_edits (gsv_key, mod_key, blob_value, install_order) \
         VALUES ('some_key', 'mod1', X'DEADBEEF', 1)",
        [],
    )
    .expect("first insert must succeed");

    let result = conn.execute(
        "INSERT INTO gsv_edits (gsv_key, mod_key, blob_value, install_order) \
         VALUES ('some_key', 'mod1', X'CAFEBABE', 2)",
        [],
    );
    assert!(
        result.is_err(),
        "duplicate (gsv_key, mod_key) must be rejected"
    );
}

// ─── 11. FK: file_owners rejects unknown mod_key ─────────────────────────

#[test]
fn file_owners_fk_rejects_unknown_mod() {
    let conn = open_db();
    let result = conn.execute(
        "INSERT INTO file_owners (file_path, mod_key, install_order) \
         VALUES ('Data/test.dds', 'nonexistent', 1)",
        [],
    );
    assert!(
        result.is_err(),
        "file_owners must reject mod_key not present in mods"
    );
}

// ─── 12. FK: ini_edits rejects unknown mod_key ───────────────────────────

#[test]
fn ini_edits_fk_rejects_unknown_mod() {
    let conn = open_db();
    let result = conn.execute(
        "INSERT INTO ini_edits (ini_file, section, key, mod_key, value, install_order) \
         VALUES ('skyrim.ini', 'Display', 'fVal', 'ghost', '1', 1)",
        [],
    );
    assert!(
        result.is_err(),
        "ini_edits must reject mod_key not present in mods"
    );
}

// ─── 13. FK: gsv_edits rejects unknown mod_key ───────────────────────────

#[test]
fn gsv_edits_fk_rejects_unknown_mod() {
    let conn = open_db();
    let result = conn.execute(
        "INSERT INTO gsv_edits (gsv_key, mod_key, blob_value, install_order) \
         VALUES ('k', 'ghost', NULL, 1)",
        [],
    );
    assert!(
        result.is_err(),
        "gsv_edits must reject mod_key not present in mods"
    );
}

// ─── 14. CASCADE: DELETE mod removes file_owners rows ─────────────────────

#[test]
fn cascade_delete_file_owners() {
    let conn = open_db();
    conn.execute(
        "INSERT INTO mods (mod_key, archive_path, name) VALUES ('mod1', 'a.7z', 'A')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO file_owners (file_path, mod_key, install_order) \
         VALUES ('Data/t.dds', 'mod1', 1)",
        [],
    )
    .unwrap();

    conn.execute("DELETE FROM mods WHERE mod_key = 'mod1'", [])
        .unwrap();

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM file_owners WHERE mod_key = 'mod1'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 0, "CASCADE must have deleted file_owners rows");
}

// ─── 15. CASCADE: DELETE mod removes ini_edits rows ───────────────────────

#[test]
fn cascade_delete_ini_edits() {
    let conn = open_db();
    conn.execute(
        "INSERT INTO mods (mod_key, archive_path, name) VALUES ('mod1', 'a.7z', 'A')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO ini_edits (ini_file, section, key, mod_key, value, install_order) \
         VALUES ('s.ini', 'S', 'K', 'mod1', 'V', 1)",
        [],
    )
    .unwrap();

    conn.execute("DELETE FROM mods WHERE mod_key = 'mod1'", [])
        .unwrap();

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM ini_edits WHERE mod_key = 'mod1'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 0, "CASCADE must have deleted ini_edits rows");
}

// ─── 16. CASCADE: DELETE mod removes gsv_edits rows ───────────────────────

#[test]
fn cascade_delete_gsv_edits() {
    let conn = open_db();
    conn.execute(
        "INSERT INTO mods (mod_key, archive_path, name) VALUES ('mod1', 'a.7z', 'A')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO gsv_edits (gsv_key, mod_key, blob_value, install_order) \
         VALUES ('k', 'mod1', X'FF', 1)",
        [],
    )
    .unwrap();

    conn.execute("DELETE FROM mods WHERE mod_key = 'mod1'", [])
        .unwrap();

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM gsv_edits WHERE mod_key = 'mod1'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 0, "CASCADE must have deleted gsv_edits rows");
}

// ─── 17. COLLATE NOCASE on file_path ──────────────────────────────────────

#[test]
fn file_path_collate_nocase() {
    let conn = open_db();
    conn.execute(
        "INSERT INTO mods (mod_key, archive_path, name) VALUES ('mod1', 'a.7z', 'A')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO file_owners (file_path, mod_key, install_order) \
         VALUES ('Data/Textures/Test.DDS', 'mod1', 1)",
        [],
    )
    .expect("insert must succeed");

    // Query with different case — must find the row.
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM file_owners WHERE file_path = 'data/textures/test.dds'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 1, "COLLATE NOCASE must match case-insensitively");

    // Same path different case + same mod_key must be rejected by PK.
    let result = conn.execute(
        "INSERT INTO file_owners (file_path, mod_key, install_order) \
         VALUES ('DATA/TEXTURES/TEST.DDS', 'mod1', 2)",
        [],
    );
    assert!(
        result.is_err(),
        "COLLATE NOCASE must treat case-variant paths as duplicates in PK"
    );
}

// ─── 18. COLLATE NOCASE on ini_edits ──────────────────────────────────────

#[test]
fn ini_edits_collate_nocase() {
    let conn = open_db();
    conn.execute(
        "INSERT INTO mods (mod_key, archive_path, name) VALUES ('mod1', 'a.7z', 'A')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO ini_edits (ini_file, section, key, mod_key, value, install_order) \
         VALUES ('Skyrim.ini', 'Display', 'fShadowDist', 'mod1', '1500', 1)",
        [],
    )
    .expect("insert must succeed");

    // Query with all-lowercase — must match.
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM ini_edits \
             WHERE ini_file = 'skyrim.ini' AND section = 'display' AND key = 'fshadowdist'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(
        count, 1,
        "ini_edits COLLATE NOCASE must match case-insensitively"
    );
}

// ─── 19. apply() rejects future schema version ───────────────────────────

#[test]
fn apply_rejects_future_schema_version() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();

    // Manually seed schema_meta with a future version.
    conn.execute_batch(
        "CREATE TABLE schema_meta (key TEXT PRIMARY KEY, int_value INTEGER, text_value TEXT);",
    )
    .unwrap();
    conn.execute(
        "INSERT INTO schema_meta (key, int_value) VALUES ('schema_version', 9999)",
        [],
    )
    .unwrap();

    let result = nmm_install_log::schema::apply(&conn);
    assert!(result.is_err(), "apply must reject a future schema version");

    match result.unwrap_err() {
        nmm_install_log::error::InstallLogError::UnsupportedSchemaVersion { found, max } => {
            assert_eq!(found, 9999);
            assert_eq!(max, nmm_install_log::schema::CURRENT_VERSION);
        }
        other => panic!("expected UnsupportedSchemaVersion, got: {:?}", other),
    }
}

// ─── 20. Multiple mods can own the same file; stack ordering works ────────

#[test]
fn multiple_mods_can_own_same_file() {
    let conn = open_db();
    conn.execute(
        "INSERT INTO mods (mod_key, archive_path, name) VALUES ('mod1', 'a.7z', 'A')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO mods (mod_key, archive_path, name) VALUES ('mod2', 'b.7z', 'B')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO file_owners (file_path, mod_key, install_order) \
         VALUES ('Data/test.dds', 'mod1', 1)",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO file_owners (file_path, mod_key, install_order) \
         VALUES ('Data/test.dds', 'mod2', 2)",
        [],
    )
    .unwrap();

    // Current owner = highest install_order.
    let owner: String = conn
        .query_row(
            "SELECT mod_key FROM file_owners WHERE file_path = 'Data/test.dds' \
             ORDER BY install_order DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(owner, "mod2", "mod2 installed last so it is current owner");

    // Previous owner = second-highest install_order.
    let prev: String = conn
        .query_row(
            "SELECT mod_key FROM file_owners WHERE file_path = 'Data/test.dds' \
             ORDER BY install_order DESC LIMIT 1 OFFSET 1",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(prev, "mod1", "mod1 installed first so it is previous owner");
}
