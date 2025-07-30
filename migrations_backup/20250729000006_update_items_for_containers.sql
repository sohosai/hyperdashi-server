-- Add container support to items table
ALTER TABLE items ADD COLUMN container_id TEXT;
ALTER TABLE items ADD COLUMN storage_type TEXT DEFAULT 'location' CHECK (storage_type IN ('location', 'container'));

-- Create foreign key constraint to containers table
CREATE INDEX idx_items_container_id ON items(container_id);

-- Update existing items to use 'location' storage type (since they currently use locations)
UPDATE items SET storage_type = 'location' WHERE storage_type IS NULL;