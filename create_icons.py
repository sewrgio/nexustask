import struct
import zlib

def create_dummy_png(filename):
    # 1x1 pixel black PNG
    width = 1
    height = 1
    pixel_data = b'\x00\x00\x00\xff' # RGBA black
    
    # PNG signature
    png = b'\x89PNG\r\n\x1a\n'
    
    # IHDR chunk
    ihdr = struct.pack('>IIBBBBB', width, height, 8, 6, 0, 0, 0)
    png += struct.pack('>I', 13) + b'IHDR' + ihdr + struct.pack('>I', zlib.crc32(b'IHDR' + ihdr) & 0xffffffff)
    
    # IDAT chunk
    data = b'\x00' + pixel_data # scanline filter 0
    idat = zlib.compress(data)
    png += struct.pack('>I', len(idat)) + b'IDAT' + idat + struct.pack('>I', zlib.crc32(b'IDAT' + idat) & 0xffffffff)
    
    # IEND chunk
    png += struct.pack('>I', 0) + b'IEND' + struct.pack('>I', zlib.crc32(b'IEND') & 0xffffffff)
    
    with open(filename, 'wb') as f:
        f.write(png)

import os
os.makedirs('src-tauri/icons', exist_ok=True)
create_dummy_png('src-tauri/icons/icon.png')
create_dummy_png('src-tauri/icons/32x32.png')
create_dummy_png('src-tauri/icons/128x128.png')
create_dummy_png('src-tauri/icons/128x128@2x.png')
print("Icons created")
