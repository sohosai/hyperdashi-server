-- Add container support to items table
-- Note: container_id and storage_type are already defined in the initial items table creation
-- So we just need to ensure the check constraint exists
DO $$ 
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.constraint_column_usage 
        WHERE table_name = 'items' AND column_name = 'storage_type' AND constraint_name LIKE '%check%'
    ) THEN
        ALTER TABLE items ADD CONSTRAINT items_storage_type_check CHECK (storage_type IN ('location', 'container'));
    END IF;
END $$;

-- Create foreign key constraint to containers table
CREATE INDEX IF NOT EXISTS idx_items_container_id ON items(container_id);

-- Update existing items to use 'location' storage type (since they currently use locations)
UPDATE items SET storage_type = 'location' WHERE storage_type IS NULL;