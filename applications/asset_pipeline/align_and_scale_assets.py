import bpy
import os
import math

glb_source = r"C:\Users\dasha\Downloads\glb mach1.glb"
gltf_source = r"C:\Users\dasha\Downloads\new data.gltf"

glb_dest = r"C:\Users\dasha\Documents\ascii_sprite_sheet\guinea-pig-trench-portal\glb_mach1.glb"
gltf_dest = r"C:\Users\dasha\Documents\ascii_sprite_sheet\guinea-pig-trench-portal\new_data.gltf"

aetherion_dir = r"C:\Users\dasha\Projects\aetherion-continuum"

# --- 1. Align and Save GLB ---
bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.import_scene.gltf(filepath=glb_source)

# Select all imported meshes
bpy.ops.object.select_all(action='SELECT')

# Rotate 90 degrees around X-axis (and 180 around Z-axis if needed) to align horizontally
bpy.ops.transform.rotate(value=math.radians(-90), orient_axis='X')
bpy.ops.transform.rotate(value=math.radians(180), orient_axis='Z')

# Apply rotation transformations
bpy.ops.object.transform_apply(location=False, rotation=True, scale=False)

# Export GLB to destination
bpy.ops.export_scene.gltf(filepath=glb_dest, export_format='GLB', use_selection=False)
print("GLB model aligned and exported successfully.")

# --- 2. Scale and Save GLTF ---
bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.import_scene.gltf(filepath=gltf_source)

# Select all imported meshes
bpy.ops.object.select_all(action='SELECT')

# Scale larger by 1.8x
bpy.ops.transform.resize(value=(1.8, 1.8, 1.8))

# Apply scale transformations
bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)

# Export GLTF back to destination
bpy.ops.export_scene.gltf(filepath=gltf_dest, export_format='GLTF_SEPARATE' if gltf_dest.endswith('.gltf') else 'GLB', use_selection=False)
print("GLTF model scaled larger and exported successfully.")

# --- 3. Sync to Aetherion Continuum server folder ---
import shutil
shutil.copy(glb_dest, os.path.join(aetherion_dir, "glb_mach1.glb"))
# Copy gltf and its bin file if separate
gltf_bin_src = gltf_dest.replace(".gltf", ".bin")
if os.path.exists(gltf_bin_src):
    shutil.copy(gltf_dest, os.path.join(aetherion_dir, "new_data.gltf"))
    shutil.copy(gltf_bin_src, os.path.join(aetherion_dir, "new_data.bin"))
else:
    # If exported as GLB format in dest
    shutil.copy(gltf_dest, os.path.join(aetherion_dir, "new_data.glb"))

print("Sync completed successfully.")
