-- Drop existing tables that may have been created with incompatible schemas
-- Use IF EXISTS to avoid errors if tables don't exist
DROP TABLE IF EXISTS loans;
DROP TABLE IF EXISTS items;
DROP TABLE IF EXISTS containers;
DROP TABLE IF EXISTS label_counter;
DROP TABLE IF EXISTS cable_colors;