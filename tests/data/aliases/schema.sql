CREATE TABLE meta (key text not null, value text not null);
CREATE TABLE log (ts timestamp not null default current_timestamp, v text not null, msg text not null);
CREATE TABLE seqalias (
    seqalias_id integer primary key,
    seq_id text not null,
    namespace text not null,
    alias text not null,
    added timestamp not null default current_timestamp,
    is_current int not null default 1
);
CREATE TABLE IF NOT EXISTS "yoyo_lock" (
            locked INT DEFAULT 1,
            ctime TIMESTAMP,
            pid INT NOT NULL,
            PRIMARY KEY (locked)
        );
CREATE TABLE IF NOT EXISTS "_yoyo_log" ( id VARCHAR(36), migration_hash VARCHAR(64), migration_id VARCHAR(255), operation VARCHAR(10), username VARCHAR(255), hostname VARCHAR(255), comment VARCHAR(255), created_at_utc TIMESTAMP, PRIMARY KEY (id));
CREATE TABLE IF NOT EXISTS "_yoyo_version" (version INT NOT NULL PRIMARY KEY, installed_at_utc TIMESTAMP);
CREATE TABLE IF NOT EXISTS "_yoyo_migration" ( migration_hash VARCHAR(64), migration_id VARCHAR(255), applied_at_utc TIMESTAMP, PRIMARY KEY (migration_hash));
CREATE UNIQUE INDEX meta_key_idx on meta(key);
CREATE UNIQUE INDEX seqalias_unique_ns_alias_idx on seqalias(namespace, alias) where is_current = 1
;
CREATE INDEX seqalias_seq_id_idx on seqalias(seq_id)
;
CREATE INDEX seqalias_namespace_idx on seqalias(namespace)
;
CREATE INDEX seqalias_alias_idx on seqalias(alias)
;
