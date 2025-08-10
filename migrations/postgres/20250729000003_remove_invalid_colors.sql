-- Remove invalid colors: pink and skyblue (PostgreSQL compatible)
-- These colors were mentioned as non-existent by the user

-- Remove any items that reference these colors
-- PostgreSQL version using JSONB operations
UPDATE items 
SET cable_color_pattern = (
    SELECT jsonb_agg(color)::text
    FROM jsonb_array_elements_text(cable_color_pattern::jsonb) AS color
    WHERE color != 'pink'
)
WHERE cable_color_pattern IS NOT NULL 
  AND cable_color_pattern != '[]' 
  AND cable_color_pattern != ''
  AND cable_color_pattern::jsonb ? 'pink';

UPDATE items 
SET cable_color_pattern = (
    SELECT jsonb_agg(color)::text
    FROM jsonb_array_elements_text(cable_color_pattern::jsonb) AS color
    WHERE color != 'skyblue'
)
WHERE cable_color_pattern IS NOT NULL 
  AND cable_color_pattern != '[]' 
  AND cable_color_pattern != ''
  AND cable_color_pattern::jsonb ? 'skyblue';

-- Remove the colors from cable_colors table
DELETE FROM cable_colors WHERE name IN ('pink', 'skyblue');