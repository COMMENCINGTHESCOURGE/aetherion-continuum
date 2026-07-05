import bpy
import os
import shutil
import math

gltf_source = r"C:\Users\dasha\Downloads\new data.gltf"
gltf_dest = r"C:\Users\dasha\Documents\ascii_sprite_sheet\guinea-pig-trench-portal\new_data.gltf"
aetherion_dir = r"C:\Users\dasha\Projects\aetherion-continuum"

# 1. Restore original model from Downloads (clean copy)
shutil.copy(gltf_source, gltf_dest)
bin_source = gltf_source.replace(".gltf", ".bin")
bin_dest = gltf_dest.replace(".gltf", ".bin")
if os.path.exists(bin_source):
    shutil.copy(bin_source, bin_dest)

# 2. Start Blender and apply smoothing safely without parent hierarchy modifications
bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.import_scene.gltf(filepath=gltf_dest)

# Smooth mesh normals without altering positioning/hierarchy
for obj in bpy.context.scene.objects:
    if obj.type == 'MESH':
        # Select and set active
        bpy.ops.object.select_all(action='DESELECT')
        obj.select_set(True)
        bpy.context.view_layer.objects.active = obj
        
        # Apply EEVEE normal auto-smoothing
        if hasattr(obj.data, "use_auto_smooth"):
            obj.data.use_auto_smooth = True
            obj.data.auto_smooth_angle = math.radians(30)
        bpy.ops.object.shade_smooth()

# 3. Scale up by 1.8x on disk directly on the root objects (safely preserving relative local offsets)
# Locate root meshes (those with no parent or parent empty) and scale them
for obj in bpy.context.scene.objects:
    if obj.parent is None:
        obj.select_set(True)
        
bpy.ops.transform.resize(value=(1.8, 1.8, 1.8))
bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)

# Export clean gltf
bpy.ops.export_scene.gltf(filepath=gltf_dest, export_format='GLTF_SEPARATE', use_selection=False)
print("GLTF normal smoothing and scaling applied, relative offsets preserved successfully.")

# 4. Sync updated files to Aetherion Continuum folder
shutil.copy(gltf_dest, os.path.join(aetherion_dir, "new_data.gltf"))
if os.path.exists(bin_dest):
    shutil.copy(bin_dest, os.path.join(aetherion_dir, "new_data.bin"))

print("Sync completed successfully.")
