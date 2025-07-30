-- Remove invalid colors: pink and skyblue
-- These colors were mentioned as non-existent by the user

-- Remove any items that reference these colors
-- SQLite version using JSON operations
UPDATE items 
SET cable_color_pattern = (
    SELECT json_group_array(value)
    FROM json_each(cable_color_pattern)
    WHERE value != 'pink'
)
WHERE cable_color_pattern IS NOT NULL 
  AND cable_color_pattern != '[]' 
  AND cable_color_pattern != ''
  AND json_valid(cable_color_pattern)
  AND EXISTS (
    SELECT 1 
    FROM json_each(cable_color_pattern) 
    WHERE value = 'pink'
  );

UPDATE items 
SET cable_color_pattern = (
    SELECT json_group_array(value)
    FROM json_each(cable_color_pattern)
    WHERE value != 'skyblue'
)
WHERE cable_color_pattern IS NOT NULL 
  AND cable_color_pattern != '[]' 
  AND cable_color_pattern != ''
  AND json_valid(cable_color_pattern)
  AND EXISTS (
    SELECT 1 
    FROM json_each(cable_color_pattern) 
    WHERE value = 'skyblue'
  );

-- Remove the colors from cable_colors table
DELETE FROM cable_colors WHERE name IN ('pink', 'skyblue');