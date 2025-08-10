-- Create cable colors table (PostgreSQL compatible)
CREATE TABLE IF NOT EXISTS cable_colors (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL UNIQUE,
    hex_code VARCHAR(7),
    description TEXT,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Insert common cable colors
INSERT INTO cable_colors (name, hex_code, description) VALUES 
('赤', '#FF0000', '赤色'),
('青', '#0000FF', '青色'),
('緑', '#00FF00', '緑色'),
('黄', '#FFFF00', '黄色'),
('黒', '#000000', '黒色'),
('白', '#FFFFFF', '白色'),
('グレー', '#808080', 'グレー色'),
('オレンジ', '#FFA500', 'オレンジ色'),
('紫', '#800080', '紫色'),
('茶', '#A52A2A', '茶色'),
('ピンク', '#FFC0CB', 'ピンク色'),
('シルバー', '#C0C0C0', 'シルバー色');