import bpy
import os

gltf_path = r"C:\Users\dasha\Downloads\new data.gltf"

bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.import_scene.gltf(filepath=gltf_path)

# Ensure Part 17 is selected and active
obj = bpy.data.objects.get("Part 17")
if obj:
    bpy.context.view_layer.objects.active = obj
    obj.select_set(True)
    
    # Enter edit mode and separate by loose parts
    bpy.ops.object.mode_set(mode='EDIT')
    bpy.ops.mesh.separate(type='LOOSE')
    bpy.ops.object.mode_set(mode='OBJECT')
    
    print("--- Separated Parts ---")
    for child in bpy.context.scene.objects:
        if child.type == 'MESH':
            # Calculate bounds center
            bbox = child.bound_box
            center_x = sum([v[0] for v in bbox]) / 8.0
            center_y = sum([v[1] for v in bbox]) / 8.0
            center_z = sum([v[2] for v in bbox]) / 8.0
            print(f"Name: {child.name}, Vertices: {len(child.data.vertices)}, Center X: {center_x:.3f}, Y: {center_y:.3f}, Z: {center_z:.3f}")
else:
    print("Part 17 not found.")
