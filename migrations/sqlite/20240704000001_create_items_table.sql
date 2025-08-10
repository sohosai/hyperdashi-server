-- Create items table (PostgreSQL compatible)
CREATE TABLE IF NOT EXISTS items (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    label_id TEXT UNIQUE NOT NULL,
    model_number TEXT,
    remarks TEXT,
    purchase_year INTEGER,
    purchase_amount REAL,
    durability_years INTEGER,
    is_depreciation_target BOOLEAN DEFAULT FALSE,
    connection_names TEXT, -- JSON array
    cable_color_pattern TEXT, -- JSON array  
    storage_location TEXT, -- Single storage location
    container_id TEXT, -- Container ID reference
    storage_type TEXT DEFAULT 'location', -- 'location' or 'container'
    is_on_loan BOOLEAN DEFAULT FALSE,
    qr_code_type TEXT CHECK (qr_code_type IN ('qr', 'barcode', 'none')),
    is_disposed BOOLEAN DEFAULT FALSE,
    image_url TEXT,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_items_label_id ON items(label_id);
CREATE INDEX IF NOT EXISTS idx_items_name ON items(name);
CREATE INDEX IF NOT EXISTS idx_items_is_on_loan ON items(is_on_loan);
CREATE INDEX IF NOT EXISTS idx_items_is_disposed ON items(is_disposed);