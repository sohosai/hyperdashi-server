-- Create loans table (PostgreSQL compatible)
CREATE TABLE IF NOT EXISTS loans (
    id BIGSERIAL PRIMARY KEY,
    item_id BIGINT NOT NULL REFERENCES items(id),
    student_number TEXT NOT NULL,
    student_name TEXT NOT NULL,
    organization TEXT,
    loan_date TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    return_date TIMESTAMPTZ,
    remarks TEXT,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_loans_item_id ON loans(item_id);
CREATE INDEX IF NOT EXISTS idx_loans_student_number ON loans(student_number);
CREATE INDEX IF NOT EXISTS idx_loans_return_date ON loans(return_date);