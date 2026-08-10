-- scripts/indexes_dev.sql
-- Performance indexes for the `genshin_map` schema (PostgreSQL 15).
--
-- Index gaps identified in the db_audit.md audit (P2): history (460K rows)
-- filters by creator_id / edit_type and defaults to ORDER BY update_time DESC
-- with no backing index; sys_user_device / sys_user_invitation /
-- sys_action_log / marker_item_link (707K rows) are filtered on unindexed
-- columns.
--
-- Idempotent: every statement uses CREATE INDEX IF NOT EXISTS, so this file
-- can be re-run safely at any time.
--
-- Where it runs:
--   1. local / e2e databases: applied automatically by `cargo run --bin
--      init_db` (init_db.rs embeds this file via include_str!).
--   2. production database: run once manually by ops, e.g.:
--        psql "postgres://<user>:<pass>@<host>:<port>/genshin_map" \
--          -f scripts/indexes_dev.sql
--      (init_db is never pointed at the production DB; the CREATE TABLE pass
--      would be skipped there anyway, and the indexes below are exactly what
--      production needs.)

-- history: per-creator filters, per-edit_type filters, and the default
-- ORDER BY update_time DESC used by history.rs list queries.
CREATE INDEX IF NOT EXISTS idx_history_creator_id ON genshin_map.history (creator_id);
CREATE INDEX IF NOT EXISTS idx_history_edit_type ON genshin_map.history (edit_type);
CREATE INDEX IF NOT EXISTS idx_history_update_time ON genshin_map.history (update_time DESC);

-- sys_user_device: login registration / access-policy checks are keyed on
-- (user_id, last_login_time).
CREATE INDEX IF NOT EXISTS idx_sys_user_device_user_last_login
    ON genshin_map.sys_user_device (user_id, last_login_time);

-- sys_user_invitation: code lookup on consume, creator_id for listing.
CREATE INDEX IF NOT EXISTS idx_sys_user_invitation_code ON genshin_map.sys_user_invitation (code);
CREATE INDEX IF NOT EXISTS idx_sys_user_invitation_creator_id
    ON genshin_map.sys_user_invitation (creator_id);

-- sys_action_log: per-user filters and create_time range scans.
CREATE INDEX IF NOT EXISTS idx_sys_action_log_user_id ON genshin_map.sys_action_log (user_id);
CREATE INDEX IF NOT EXISTS idx_sys_action_log_create_time ON genshin_map.sys_action_log (create_time);

-- marker_item_link (707K rows): standalone lookups/joins on either side.
-- (The (item_id, marker_id) composite index already exists; these two cover
-- queries that filter on one side only.)
CREATE INDEX IF NOT EXISTS idx_marker_item_link_item_id ON genshin_map.marker_item_link (item_id);
CREATE INDEX IF NOT EXISTS idx_marker_item_link_marker_id ON genshin_map.marker_item_link (marker_id);
