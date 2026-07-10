import bpy
import os

model_path = r"C:\Users\dasha\Documents\ascii_sprite_sheet\guinea-pig-trench-portal\glb_mach1.glb"

# Clear scene
bpy.ops.wm.read_factory_settings(use_empty=True)

# Import GLB
bpy.ops.import_scene.gltf(filepath=model_path)

# Select all meshes and scale them by 1.5
bpy.ops.object.select_all(action='SELECT')
bpy.ops.transform.resize(value=(1.5, 1.5, 1.5))

# Apply the scale transformation directly to the vertices (freeze transformation)
bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)

# Export back to the same path
bpy.ops.export_scene.gltf(
    filepath=model_path,
    export_format='GLB',
    use_selection=False
)

print("GLB model scaled by 1.5x and transformations applied successfully.")
