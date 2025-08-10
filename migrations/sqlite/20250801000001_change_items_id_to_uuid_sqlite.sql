-- SQLite version: Change items ID to UUID (stored as TEXT)
-- Note: SQLite doesn't have native UUID type, so we use TEXT

-- Create new tables with UUID as TEXT
CREATE TABLE items_new (
    id TEXT PRIMARY KEY DEFAULT (lower(
        hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-' || '4' || 
        substr(hex(randomblob(2)), 2) || '-' || 
        substr('89ab', 1 + (abs(random()) % 4), 1) ||
        substr(hex(randomblob(2)), 2) || '-' || hex(randomblob(6))
    )),
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
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE loans_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    item_id TEXT NOT NULL,
    student_number TEXT NOT NULL,
    student_name TEXT NOT NULL,
    organization TEXT,
    loan_date TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    return_date TEXT,
    remarks TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (item_id) REFERENCES items_new(id) ON DELETE CASCADE
);

-- Create temporary mapping table
CREATE TEMPORARY TABLE id_mapping (
    old_id INTEGER,
    new_id TEXT
);

-- Generate UUIDs for existing items
INSERT INTO id_mapping (old_id, new_id)
SELECT id, lower(
    hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-' || '4' || 
    substr(hex(randomblob(2)), 2) || '-' || 
    substr('89ab', 1 + (abs(random()) % 4), 1) ||
    substr(hex(randomblob(2)), 2) || '-' || hex(randomblob(6))
) FROM items;

-- Copy items with new UUIDs
INSERT INTO items_new (
    id, name, label_id, model_number, remarks, purchase_year,
    purchase_amount, durability_years, is_depreciation_target, connection_names,
    cable_color_pattern, storage_location, container_id, storage_type, 
    is_on_loan, qr_code_type, is_disposed, image_url, created_at, updated_at
)
SELECT 
    m.new_id, i.name, i.label_id, i.model_number, i.remarks, i.purchase_year,
    i.purchase_amount, i.durability_years, i.is_depreciation_target, i.connection_names,
    i.cable_color_pattern, i.storage_location, i.container_id, i.storage_type,
    i.is_on_loan, i.qr_code_type, i.is_disposed, i.image_url, i.created_at, i.updated_at
FROM items i
JOIN id_mapping m ON i.id = m.old_id;

-- Copy loans with new item_id references
INSERT INTO loans_new (
    id, item_id, student_number, student_name, organization,
    loan_date, return_date, remarks, created_at, updated_at
)
SELECT 
    l.id, m.new_id, l.student_number, l.student_name, l.organization,
    l.loan_date, l.return_date, l.remarks, l.created_at, l.updated_at
FROM loans l
JOIN id_mapping m ON l.item_id = m.old_id;

-- Drop old tables
DROP TABLE loans;
DROP TABLE items;

-- Rename new tables
ALTER TABLE items_new RENAME TO items;
ALTER TABLE loans_new RENAME TO loans;

-- Recreate indexes
CREATE INDEX idx_items_label_id ON items(label_id);
CREATE INDEX idx_items_name ON items(name);
CREATE INDEX idx_items_is_on_loan ON items(is_on_loan);
CREATE INDEX idx_items_is_disposed ON items(is_disposed);
CREATE INDEX idx_loans_item_id ON loans(item_id);
CREATE INDEX idx_loans_student_number ON loans(student_number);
CREATE INDEX idx_loans_return_date ON loans(return_date);