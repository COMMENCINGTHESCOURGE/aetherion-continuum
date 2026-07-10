import bpy
import os
import math

# Paths
glb_path = r"C:\Users\dasha\Downloads\glb mach1.glb"
gltf_path = r"C:\Users\dasha\Downloads\new data.gltf"
output_path = r"C:\Users\dasha\.gemini\antigravity-ide\brain\43b6fe1c-0cbf-4636-9f3d-29dc0e11cb9c\glider_comparison_side_by_side.png"

# Clear default objects
bpy.ops.wm.read_factory_settings(use_empty=True)

# --- 1. Import GLB (Left Model) ---
bpy.ops.import_scene.gltf(filepath=glb_path)
glb_objs = [obj for obj in bpy.context.scene.objects if obj.type == 'MESH']

# Create parent empty for left model
glb_parent = bpy.data.objects.new("GLB_Parent", None)
bpy.context.scene.collection.objects.link(glb_parent)
for obj in glb_objs:
    if obj.parent is None:
        obj.parent = glb_parent

# Position GLB on the left, scaled by 1.5x (to match the scale script) and pointing forward
glb_parent.location = (-2.5, 0, 0)
glb_parent.rotation_euler = (0, 0, 0)
glb_parent.scale = (1.5, 1.5, 1.5)

# Deselect all for next import
bpy.ops.object.select_all(action='DESELECT')

# --- 2. Import GLTF (Right Model) ---
bpy.ops.import_scene.gltf(filepath=gltf_path)
gltf_objs = [obj for obj in bpy.context.scene.objects if obj not in glb_objs and obj.type == 'MESH']

# Create parent empty for right model
gltf_parent = bpy.data.objects.new("GLTF_Parent", None)
bpy.context.scene.collection.objects.link(gltf_parent)
for obj in gltf_objs:
    if obj.parent is None:
        obj.parent = gltf_parent

# Position GLTF on the right, scaled matching game proportions
gltf_parent.location = (2.5, 0, 0)
gltf_parent.rotation_euler = (0, 0, 0)
gltf_parent.scale = (1.5, 1.5, 1.5)

# --- 3. Camera Setup ---
camera_data = bpy.data.cameras.new(name="Camera")
camera_object = bpy.data.objects.new("Camera", camera_data)
bpy.context.scene.collection.objects.link(camera_object)
bpy.context.scene.camera = camera_object

# Position camera looking directly at the center of both models
camera_object.location = (0, -7.5, 1.8)
camera_object.rotation_euler = (math.radians(82), 0, 0)

# --- 4. Lighting Setup ---
# Main studio key light
light_data_1 = bpy.data.lights.new(name="KeyLight", type='SUN')
light_data_1.energy = 4.0
light_data_1.color = (0.2, 0.8, 1.0) # Teal tinted key
light_object_1 = bpy.data.objects.new("KeyLight", light_data_1)
bpy.context.scene.collection.objects.link(light_object_1)
light_object_1.rotation_euler = (math.radians(45), 0, math.radians(45))

# Fill light from opposite side
light_data_2 = bpy.data.lights.new(name="FillLight", type='SUN')
light_data_2.energy = 2.0
light_data_2.color = (0.8, 0.2, 1.0) # Magenta tinted fill
light_object_2 = bpy.data.objects.new("FillLight", light_data_2)
bpy.context.scene.collection.objects.link(light_object_2)
light_object_2.rotation_euler = (math.radians(-45), 0, math.radians(-135))

# --- 5. Render settings ---
bpy.context.scene.render.engine = 'BLENDER_EEVEE_NEXT' if hasattr(bpy.context.scene.render, "BLENDER_EEVEE_NEXT") else 'BLENDER_EEVEE'
bpy.context.scene.render.resolution_x = 1024
bpy.context.scene.render.resolution_y = 768
bpy.context.scene.render.image_settings.file_format = 'PNG'
bpy.context.scene.render.filepath = output_path

# Execute render
bpy.ops.render.render(write_still=True)
print("Comparison render completed successfully.")
