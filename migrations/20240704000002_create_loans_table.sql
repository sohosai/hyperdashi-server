-- Create loans table (compatible with both PostgreSQL and SQLite)
CREATE TABLE IF NOT EXISTS loans (
    id INTEGER PRIMARY KEY,
    item_id INTEGER NOT NULL REFERENCES items(id),
    student_number TEXT NOT NULL,
    student_name TEXT NOT NULL,
    organization TEXT,
    loan_date TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    return_date TIMESTAMP,
    remarks TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_loans_item_id ON loans(item_id);
CREATE INDEX IF NOT EXISTS idx_loans_student_number ON loans(student_number);
CREATE INDEX IF NOT EXISTS idx_loans_return_date ON loans(return_date);