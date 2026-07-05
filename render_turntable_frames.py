import bpy
import os
import math

gltf_path = r"C:\Users\dasha\Documents\ascii_sprite_sheet\guinea-pig-trench-portal\new_data.gltf"
output_dir = r"C:\Users\dasha\.gemini\antigravity-ide\brain\43b6fe1c-0cbf-4636-9f3d-29dc0e11cb9c"
temp_frame_dir = os.path.join(output_dir, "temp_frames")
os.makedirs(temp_frame_dir, exist_ok=True)

# Clear factory scene
bpy.ops.wm.read_factory_settings(use_empty=True)

# Import GLTF
bpy.ops.import_scene.gltf(filepath=gltf_path)

# Create parent empty for turntable rotation
model_parent = bpy.data.objects.new("ModelParent", None)
bpy.context.scene.collection.objects.link(model_parent)
for obj in bpy.context.scene.objects:
    if obj.type == 'MESH' and obj.parent is None:
        obj.parent = model_parent

# Focus camera setup
camera_data = bpy.data.cameras.new(name="Camera")
camera_object = bpy.data.objects.new("Camera", camera_data)
bpy.context.scene.collection.objects.link(camera_object)
bpy.context.scene.camera = camera_object
camera_object.location = (0, -7.5, 1.8)
camera_object.rotation_euler = (math.radians(82), 0, 0)

# Setup lights
light_data_1 = bpy.data.lights.new(name="TealSun", type='SUN')
light_data_1.energy = 4.0
light_data_1.color = (0.0, 1.0, 0.8) # Cyber teal
light_object_1 = bpy.data.objects.new("TealSun", light_data_1)
bpy.context.scene.collection.objects.link(light_object_1)
light_object_1.rotation_euler = (math.radians(45), 0, math.radians(45))

light_data_2 = bpy.data.lights.new(name="PinkSun", type='SUN')
light_data_2.energy = 2.0
light_data_2.color = (1.0, 0.0, 0.8) # Hot pink/magenta
light_object_2 = bpy.data.objects.new("PinkSun", light_data_2)
bpy.context.scene.collection.objects.link(light_object_2)
light_object_2.rotation_euler = (math.radians(-45), 0, math.radians(-135))

# Set render settings
bpy.context.scene.render.engine = 'BLENDER_EEVEE_NEXT' if hasattr(bpy.context.scene.render, "BLENDER_EEVEE_NEXT") else 'BLENDER_EEVEE'
bpy.context.scene.render.resolution_x = 480
bpy.context.scene.render.resolution_y = 360
bpy.context.scene.render.image_settings.file_format = 'PNG'

# Render 30 frames for a smooth loop
num_frames = 30
for f in range(num_frames):
    angle_deg = f * (360.0 / num_frames)
    model_parent.rotation_euler = (0, 0, math.radians(angle_deg))
    
    frame_path = os.path.join(temp_frame_dir, f"frame_{f:03d}.png")
    bpy.context.scene.render.filepath = frame_path
    bpy.ops.render.render(write_still=True)
    print(f"Rendered turntable frame {f}/{num_frames}")
