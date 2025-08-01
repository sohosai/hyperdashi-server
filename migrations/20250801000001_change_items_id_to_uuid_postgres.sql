-- PostgreSQL version: Change items ID to UUID

-- Create temporary mapping table
CREATE TEMPORARY TABLE id_mapping (
    old_id BIGINT,
    new_id UUID DEFAULT gen_random_uuid()
);

-- Insert mapping for all existing items
INSERT INTO id_mapping (old_id)
SELECT id FROM items;

-- Add new UUID columns
ALTER TABLE items ADD COLUMN id_new UUID;
ALTER TABLE loans ADD COLUMN item_id_new UUID;

-- Update items with new UUIDs
UPDATE items 
SET id_new = id_mapping.new_id
FROM id_mapping
WHERE items.id = id_mapping.old_id;

-- Update loans with new UUID references
UPDATE loans
SET item_id_new = id_mapping.new_id
FROM id_mapping
WHERE loans.item_id = id_mapping.old_id;

-- Drop old constraints
ALTER TABLE loans DROP CONSTRAINT IF EXISTS loans_item_id_fkey;
ALTER TABLE items DROP CONSTRAINT IF EXISTS items_pkey;

-- Drop old columns
ALTER TABLE loans DROP COLUMN item_id;
ALTER TABLE items DROP COLUMN id;

-- Rename new columns
ALTER TABLE items RENAME COLUMN id_new TO id;
ALTER TABLE loans RENAME COLUMN item_id_new TO item_id;

-- Add new constraints
ALTER TABLE items ADD PRIMARY KEY (id);
ALTER TABLE loans ADD CONSTRAINT loans_item_id_fkey FOREIGN KEY (item_id) REFERENCES items(id) ON DELETE CASCADE;

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_items_id ON items(id);
CREATE INDEX IF NOT EXISTS idx_loans_item_id ON loans(item_id);