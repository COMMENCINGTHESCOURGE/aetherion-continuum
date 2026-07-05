import bpy
import os
import math

# Paths
model_path = r"C:\Users\dasha\Documents\ascii_sprite_sheet\guinea-pig-trench-portal\glb_mach1.glb"
output_dir = r"C:\Users\dasha\.gemini\antigravity-ide\brain\43b6fe1c-0cbf-4636-9f3d-29dc0e11cb9c"

# Clear default objects
bpy.ops.wm.read_factory_settings(use_empty=True)

# Import GLB
bpy.ops.import_scene.gltf(filepath=model_path)

# Find imported objects
imported_objs = [obj for obj in bpy.context.scene.objects if obj.type == 'MESH']
if not imported_objs:
    raise Exception("No meshes imported!")

# Create a container group/empty for the model to rotate it easily
model_parent = bpy.data.objects.new("ModelParent", None)
bpy.context.scene.collection.objects.link(model_parent)
for obj in imported_objs:
    if obj.parent is None:
        obj.parent = model_parent

# Focus camera setup
camera_data = bpy.data.cameras.new(name="Camera")
camera_object = bpy.data.objects.new("Camera", camera_data)
bpy.context.scene.collection.objects.link(camera_object)
bpy.context.scene.camera = camera_object

# Move camera back
camera_object.location = (0, -8, 2)
camera_object.rotation_euler = (math.radians(80), 0, 0)

# Setup lights
light_data_1 = bpy.data.lights.new(name="CyanLight", type='SUN')
light_data_1.energy = 5.0
light_object_1 = bpy.data.objects.new("CyanLight", light_data_1)
bpy.context.scene.collection.objects.link(light_object_1)
light_object_1.rotation_euler = (math.radians(45), 0, math.radians(45))

light_data_2 = bpy.data.lights.new(name="MagentaLight", type='SUN')
light_data_2.energy = 3.0
light_object_2 = bpy.data.objects.new("MagentaLight", light_data_2)
bpy.context.scene.collection.objects.link(light_object_2)
light_object_2.rotation_euler = (math.radians(-45), 0, math.radians(-135))

# Set render resolution and engine
bpy.context.scene.render.engine = 'BLENDER_EEVEE_NEXT' if hasattr(bpy.context.scene.render, "BLENDER_EEVEE_NEXT") else 'BLENDER_EEVEE'
bpy.context.scene.render.resolution_x = 800
bpy.context.scene.render.resolution_y = 600
bpy.context.scene.render.image_settings.file_format = 'PNG'

# Render 16 different angles/lighting configurations
num_angles = 16
for i in range(num_angles):
    angle_deg = i * (360.0 / num_angles)
    angle_rad = math.radians(angle_deg)
    
    # Rotate model parent
    model_parent.rotation_euler = (0, 0, angle_rad)
    
    # Alternate light colors and intensities to show texture & forms
    if i % 3 == 0:
        light_data_1.color = (0.0, 1.0, 1.0) # Cyan
        light_data_2.color = (1.0, 0.0, 1.0) # Magenta
    elif i % 3 == 1:
        light_data_1.color = (1.0, 0.8, 0.2) # Gold
        light_data_2.color = (0.2, 0.4, 1.0) # Deep Blue
    else:
        light_data_1.color = (1.0, 1.0, 1.0) # White key
        light_data_2.color = (0.5, 0.5, 0.6) # Cool fill
        
    # Orbit camera vertically slightly as well
    cam_z = 1.0 + 3.0 * math.sin(math.radians(i * 45))
    camera_object.location = (0, -8, cam_z)
    camera_object.rotation_euler = (math.radians(90 - (cam_z * 5)), 0, 0)
    
    # Render path
    output_path = os.path.join(output_dir, f"mach1_render_angle_{i}.png")
    bpy.context.scene.render.filepath = output_path
    
    # Execute render
    bpy.ops.render.render(write_still=True)
    print(f"Rendered angle {i}/{num_angles}: {output_path}")

print("Batch rendering completed successfully.")
