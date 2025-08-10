-- Create label counter table for persistent ID tracking (PostgreSQL compatible)
CREATE TABLE IF NOT EXISTS label_counter (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    current_value INTEGER NOT NULL DEFAULT 0
);

-- Initialize with 0 (will generate 0000 as first ID)
INSERT INTO label_counter (id, current_value) VALUES (1, 0);