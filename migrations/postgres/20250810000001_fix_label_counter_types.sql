-- Fix label_counter column types for PostgreSQL compatibility
-- Change INTEGER to BIGINT to match Rust i64 type

-- Backup existing data and recreate table with correct types
-- First save any existing data
CREATE TEMPORARY TABLE label_counter_backup AS SELECT * FROM label_counter;

-- Drop and recreate with correct types
DROP TABLE IF EXISTS label_counter;
CREATE TABLE label_counter (
    id BIGINT PRIMARY KEY CHECK (id = 1),
    current_value BIGINT NOT NULL DEFAULT 0
);

-- Restore data with proper type casting
INSERT INTO label_counter (id, current_value) 
SELECT id::BIGINT, current_value::BIGINT FROM label_counter_backup;

-- Cleanup
DROP TABLE label_counter_backup;