-- Update cable colors to 9 colors with pink base
-- First, clear existing colors
DELETE FROM cable_colors;

-- Insert new 9 cable colors
INSERT INTO cable_colors (name, hex_code, description) VALUES 
('red', '#ff0000', '赤色'),
('yellow', '#ffff00', '黄色'),
('green', '#00bb00', '緑色'),
('blue', '#0000a4', '青色'),
('white', '#ffffff', '白色'),
('gray', '#8e8e8e', 'グレー色'),
('black', '#000000', '黒色'),
('skyblue', '#00ffff', '水色'),
('pink', '#f46ed6', 'ピンク色');