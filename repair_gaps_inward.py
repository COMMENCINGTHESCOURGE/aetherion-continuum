import bpy
import os
import shutil
import math

gltf_source = r"C:\Users\dasha\Downloads\new data.gltf"
gltf_dest = r"C:\Users\dasha\Documents\ascii_sprite_sheet\guinea-pig-trench-portal\new_data.gltf"
aetherion_dir = r"C:\Users\dasha\Projects\aetherion-continuum"

# 1. Restore clean original model
shutil.copy(gltf_source, gltf_dest)
bin_source = gltf_source.replace(".gltf", ".bin")
bin_dest = gltf_dest.replace(".gltf", ".bin")
if os.path.exists(bin_source):
    shutil.copy(bin_source, bin_dest)

# 2. Open in Blender
bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.import_scene.gltf(filepath=gltf_dest)

# Ensure Part 17 is active and separate it
obj = bpy.data.objects.get("Part 17")
if obj:
    bpy.context.view_layer.objects.active = obj
    obj.select_set(True)
    
    # Separate into loose parts
    bpy.ops.object.mode_set(mode='EDIT')
    bpy.ops.mesh.separate(type='LOOSE')
    bpy.ops.object.mode_set(mode='OBJECT')
    
    # 3. Slide wing elements inward to close the gaps
    # Shifting value of 0.021 brings the wing roots flush against the fuselage
    shift_val = 0.021
    
    for child in bpy.context.scene.objects:
        if child.type == 'MESH' and child.name.startswith("Part 17"):
            bbox = child.bound_box
            center_x = sum([v[0] for v in bbox]) / 8.0
            
            if center_x < -0.02:
                # Left wing elements go right (inward)
                child.location.x += shift_val
            elif center_x > 0.02:
                # Right wing elements go left (inward)
                child.location.x -= shift_val

    # 4. Join them back together
    bpy.ops.object.select_all(action='SELECT')
    bpy.ops.object.join()
    
    # Rename joined mesh back to Part 17
    joined_obj = bpy.context.view_layer.objects.active
    joined_obj.name = "Part 17"
    
    # 5. Apply auto-smoothing and scaling
    if hasattr(joined_obj.data, "use_auto_smooth"):
        joined_obj.data.use_auto_smooth = True
        joined_obj.data.auto_smooth_angle = math.radians(30)
    bpy.ops.object.shade_smooth()
    
    # Scale up by 1.8x
    joined_obj.select_set(True)
    bpy.ops.transform.resize(value=(1.8, 1.8, 1.8))
    bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)

# 6. Export repaired model
bpy.ops.export_scene.gltf(filepath=gltf_dest, export_format='GLTF_SEPARATE', use_selection=False)
print("GLTF model gaps repaired, scaled, and exported successfully.")

# 7. Sync to Aetherion Continuum folder
shutil.copy(gltf_dest, os.path.join(aetherion_dir, "new_data.gltf"))
if os.path.exists(bin_dest):
    shutil.copy(bin_dest, os.path.join(aetherion_dir, "new_data.bin"))
print("Sync completed.")
