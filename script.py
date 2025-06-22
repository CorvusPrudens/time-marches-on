#!/usr/bin/env python3

import os
import math
from PIL import Image
import argparse

def calculate_grid_size(width, height, grid_size=16):
    """Calculate how many grid cells a sprite needs."""
    grid_width = math.ceil(width / grid_size)
    grid_height = math.ceil(height / grid_size)
    return grid_width, grid_height

def find_position_in_grid(occupied_grid, sprite_grid_width, sprite_grid_height, max_columns):
    """Find the next available position in the grid for a sprite."""
    max_rows = len(occupied_grid)
    
    for row in range(max_rows):
        for col in range(max_columns - sprite_grid_width + 1):
            # Check if this position and required cells are free
            can_place = True
            for r in range(sprite_grid_height):
                for c in range(sprite_grid_width):
                    if row + r >= max_rows:
                        # Need to expand grid
                        return row, col, True
                    if occupied_grid[row + r][col + c]:
                        can_place = False
                        break
                if not can_place:
                    break
            
            if can_place:
                return row, col, False
    
    # If we get here, we need a new row
    return max_rows, 0, True

def mark_occupied(occupied_grid, row, col, sprite_grid_width, sprite_grid_height):
    """Mark grid cells as occupied by a sprite."""
    # Expand grid if necessary
    while len(occupied_grid) < row + sprite_grid_height:
        occupied_grid.append([False] * len(occupied_grid[0]) if occupied_grid else [False] * 20)
    
    for r in range(sprite_grid_height):
        for c in range(sprite_grid_width):
            occupied_grid[row + r][col + c] = True

def create_sprite_sheet(input_folder, output_path="sprite_sheet.png", grid_size=16, max_columns=None):
    """
    Create a sprite sheet from PNG files, aligned to a grid while preserving original dimensions.
    
    Args:
        input_folder (str): Path to folder containing PNG sprites
        output_path (str): Output path for the sprite sheet
        grid_size (int): Grid alignment size (default: 16x16)
        max_columns (int): Maximum columns in grid units (auto-calculated if None)
    """
    
    # Get all PNG files from the input folder
    png_files = []
    for filename in os.listdir(input_folder):
        if filename.lower().endswith('.png'):
            png_files.append(os.path.join(input_folder, filename))
    
    if not png_files:
        print(f"No PNG files found in {input_folder}")
        return
    
    # Sort files for consistent ordering
    png_files.sort()
    
    print(f"Found {len(png_files)} PNG files")
    
    # Load all images and calculate their grid requirements
    sprites_data = []
    max_sprite_grid_width = 0
    
    for png_file in png_files:
        try:
            img = Image.open(png_file)
            # Convert to RGBA if not already
            if img.mode != 'RGBA':
                img = img.convert('RGBA')
            
            width, height = img.size
            grid_width, grid_height = calculate_grid_size(width, height, grid_size)
            
            sprites_data.append({
                'image': img,
                'filename': os.path.basename(png_file),
                'original_size': (width, height),
                'grid_size': (grid_width, grid_height)
            })
            
            max_sprite_grid_width = max(max_sprite_grid_width, grid_width)
            
            print(f"Loaded: {os.path.basename(png_file)} ({width}x{height}px, needs {grid_width}x{grid_height} grid cells)")
            
        except Exception as e:
            print(f"Error loading {png_file}: {e}")
    
    if not sprites_data:
        print("No valid sprites loaded")
        return
    
    # Calculate grid layout
    if max_columns is None:
        # Auto-calculate columns based on largest sprite and reasonable sheet width
        max_columns = max(8, max_sprite_grid_width * 4)  # At least 8 columns or 4x the widest sprite
    
    print(f"Using grid: {max_columns} columns maximum, {grid_size}x{grid_size} pixel cells")
    
    # Place sprites in the grid
    occupied_grid = [[False] * max_columns]  # Start with one row
    sprite_positions = []
    
    for sprite_data in sprites_data:
        grid_width, grid_height = sprite_data['grid_size']
        
        # Find position for this sprite
        row, col, need_expand = find_position_in_grid(occupied_grid, grid_width, grid_height, max_columns)
        
        if need_expand:
            # Expand grid vertically
            while len(occupied_grid) < row + grid_height:
                occupied_grid.append([False] * max_columns)
        
        # Mark cells as occupied
        mark_occupied(occupied_grid, row, col, grid_width, grid_height)
        
        # Calculate pixel position
        pixel_x = col * grid_size
        pixel_y = row * grid_size
        
        sprite_positions.append({
            'sprite': sprite_data,
            'grid_pos': (row, col),
            'pixel_pos': (pixel_x, pixel_y)
        })
    
    # Calculate final sheet dimensions
    sheet_grid_height = len(occupied_grid)
    sheet_width = max_columns * grid_size
    sheet_height = sheet_grid_height * grid_size
    
    print(f"Final sprite sheet: {max_columns}x{sheet_grid_height} grid cells ({sheet_width}x{sheet_height} pixels)")
    
    # Create the sprite sheet
    sprite_sheet = Image.new('RGBA', (sheet_width, sheet_height), (0, 0, 0, 0))
    
    # Place sprites on the sheet
    for pos_data in sprite_positions:
        sprite = pos_data['sprite']['image']
        x, y = pos_data['pixel_pos']
        sprite_sheet.paste(sprite, (x, y), sprite)  # Use sprite as mask for alpha blending
        
        filename = pos_data['sprite']['filename']
        grid_row, grid_col = pos_data['grid_pos']
        print(f"Placed {filename} at grid ({grid_col}, {grid_row}) -> pixels ({x}, {y})")
    
    # Save the sprite sheet
    sprite_sheet.save(output_path)
    print(f"Sprite sheet saved as: {output_path}")
    
    # Generate metadata file
    metadata_path = output_path.replace('.png', '_metadata.txt')
    with open(metadata_path, 'w') as f:
        f.write(f"Sprite Sheet Metadata\n")
        f.write(f"=====================\n")
        f.write(f"Grid Cell Size: {grid_size}x{grid_size}\n")
        f.write(f"Grid Dimensions: {max_columns}x{sheet_grid_height} cells\n")
        f.write(f"Sheet Size: {sheet_width}x{sheet_height} pixels\n")
        f.write(f"Total Sprites: {len(sprites_data)}\n\n")
        f.write(f"Sprite Mapping:\n")
        f.write(f"---------------\n")
        f.write(f"{'Filename':<25} {'Original Size':<15} {'Grid Cells':<12} {'Grid Pos':<12} {'Pixel Pos':<12}\n")
        f.write(f"{'-'*25} {'-'*15} {'-'*12} {'-'*12} {'-'*12}\n")
        
        for i, pos_data in enumerate(sprite_positions):
            sprite_data = pos_data['sprite']
            filename = sprite_data['filename']
            orig_w, orig_h = sprite_data['original_size']
            grid_w, grid_h = sprite_data['grid_size']
            grid_row, grid_col = pos_data['grid_pos']
            pixel_x, pixel_y = pos_data['pixel_pos']
            
            f.write(f"{filename:<25} {orig_w}x{orig_h:<11} {grid_w}x{grid_h:<8} ({grid_col},{grid_row})<8 ({pixel_x},{pixel_y})\n")
    
    print(f"Metadata saved as: {metadata_path}")

def main():
    parser = argparse.ArgumentParser(description='Create a sprite sheet aligned to a grid while preserving original dimensions')
    parser.add_argument('input_folder', help='Folder containing PNG sprite files')
    parser.add_argument('-o', '--output', default='sprite_sheet.png', 
                       help='Output sprite sheet filename (default: sprite_sheet.png)')
    parser.add_argument('-g', '--grid-size', type=int, default=16,
                       help='Grid cell size in pixels (default: 16)')
    parser.add_argument('-c', '--max-columns', type=int, default=None,
                       help='Maximum columns in grid units (auto-calculated if not specified)')
    
    args = parser.parse_args()
    
    if not os.path.isdir(args.input_folder):
        print(f"Error: {args.input_folder} is not a valid directory")
        return
    
    create_sprite_sheet(args.input_folder, args.output, args.grid_size, args.max_columns)

if __name__ == "__main__":
    main()
