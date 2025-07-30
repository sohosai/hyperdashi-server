-- Create containers table
CREATE TABLE containers (
    id TEXT PRIMARY KEY,  -- Container ID (same format as item IDs)
    name TEXT NOT NULL,
    description TEXT,
    location TEXT NOT NULL,  -- Where the container is located
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    is_disposed BOOLEAN DEFAULT FALSE
);

-- Create index for location-based queries
CREATE INDEX idx_containers_location ON containers(location);
CREATE INDEX idx_containers_is_disposed ON containers(is_disposed);