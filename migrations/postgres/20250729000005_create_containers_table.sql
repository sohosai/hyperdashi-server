-- Create containers table (PostgreSQL compatible)
CREATE TABLE IF NOT EXISTS containers (
    id TEXT PRIMARY KEY,  -- Container ID (same format as item IDs)
    name TEXT NOT NULL,
    description TEXT,
    location TEXT NOT NULL,  -- Where the container is located
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    is_disposed BOOLEAN DEFAULT FALSE
);

-- Create index for location-based queries
CREATE INDEX IF NOT EXISTS idx_containers_location ON containers(location);
CREATE INDEX IF NOT EXISTS idx_containers_is_disposed ON containers(is_disposed);