-- The Rust API models deserialize connector/tag IDs as i64.
-- Align the new master tables with existing Postgres master tables
-- such as cable_colors, which use BIGSERIAL/BIGINT IDs.

ALTER TABLE item_tags
    DROP CONSTRAINT IF EXISTS item_tags_tag_id_fkey;

ALTER TABLE connectors
    ALTER COLUMN id TYPE BIGINT;

ALTER TABLE tags
    ALTER COLUMN id TYPE BIGINT;

ALTER TABLE item_tags
    ALTER COLUMN tag_id TYPE BIGINT;

ALTER TABLE item_tags
    ADD CONSTRAINT item_tags_tag_id_fkey
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE;
