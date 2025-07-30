-- Update storage_locations to be a single TEXT field, not a JSON array
-- Also, rename the column to storage_location to reflect the change

-- Step 1: Add a new temporary column for the single location
ALTER TABLE items ADD COLUMN storage_location_temp TEXT;

-- Step 2: Migrate data from the old JSON array to the new single field
-- This will vary slightly between PostgreSQL and SQLite

-- For SQLite, we can use json_extract to get the first element.
-- If json_extract is not available, this might need to be done in the application layer.
UPDATE items SET storage_location_temp = json_extract(storage_locations, '$[0]') WHERE storage_locations IS NOT NULL AND storage_locations != '[]';

-- For PostgreSQL, the syntax is different:
-- UPDATE items SET storage_location_temp = storage_locations::jsonb ->> 0 WHERE storage_locations IS NOT NULL AND storage_locations != '[]';

-- Step 3: Drop the old column
ALTER TABLE items DROP COLUMN storage_locations;

-- Step 4: Rename the new column to the final name
ALTER TABLE items RENAME COLUMN storage_location_temp TO storage_location;
