import shutil
import os

src_base = "src"
dest_base = "src-tauri/src"

items = ["application", "domain", "ports", "infrastructure", "state.rs"]

for item in items:
    src_path = os.path.join(src_base, item)
    dest_path = os.path.join(dest_base, item)
    
    if os.path.exists(dest_path):
        if os.path.isdir(dest_path):
            shutil.rmtree(dest_path)
        else:
            os.remove(dest_path)
            
    if os.path.isdir(src_path):
        shutil.copytree(src_path, dest_path)
        print(f"Copied directory {item}")
    else:
        shutil.copy2(src_path, dest_path)
        print(f"Copied file {item}")
