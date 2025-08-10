-- Add skyblue and gray colors back to cable_colors
INSERT INTO cable_colors (name, hex_code, description) VALUES 
('skyblue', '#00ffff', '水色'),
('gray', '#8e8e8e', 'グレー色')
ON CONFLICT(name) DO NOTHING;