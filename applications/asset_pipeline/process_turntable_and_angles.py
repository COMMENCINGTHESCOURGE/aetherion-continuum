import bpy
import os
import math

gltf_path = r"C:\Users\dasha\Documents\ascii_sprite_sheet\guinea-pig-trench-portal\new_data.gltf"
output_dir = r"C:\Users\dasha\.gemini\antigravity-ide\brain\43b6fe1c-0cbf-4636-9f3d-29dc0e11cb9c"

# Clear factory scene
bpy.ops.wm.read_factory_settings(use_empty=True)

# Import GLTF
bpy.ops.import_scene.gltf(filepath=gltf_path)

# 1. APPLY GEOMETRIC DESIGN ADJUSTMENTS (Sleek aerodynamic smoothing)
bpy.ops.object.select_all(action='SELECT')
for obj in bpy.context.scene.objects:
    if obj.type == 'MESH':
        bpy.context.view_layer.objects.active = obj
        # Apply auto-smooth at 30 degrees to preserve sharp feature edges while smoothing large panel surfaces
        if hasattr(obj.data, "use_auto_smooth"):
            obj.data.use_auto_smooth = True
            obj.data.auto_smooth_angle = math.radians(30)
        bpy.ops.object.shade_smooth()

# Export improved model back
bpy.ops.export_scene.gltf(filepath=gltf_path, export_format='GLTF_SEPARATE', use_selection=False)
print("Design adjustments (auto-smooth normals) applied and gltf overwritten.")

# Create parent empty for turntable
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
bpy.context.scene.render.resolution_x = 800
bpy.context.scene.render.resolution_y = 600
bpy.context.scene.render.image_settings.file_format = 'PNG'

# --- 2. Render 12 Photos ---
num_photos = 12
for i in range(num_photos):
    angle_deg = i * (360.0 / num_photos)
    model_parent.rotation_euler = (0, 0, math.radians(angle_deg))
    
    # Vertically animate camera orbit elevation
    cam_z = 0.5 + 2.5 * math.sin(math.radians(i * 60))
    camera_object.location = (0, -7.5, cam_z)
    camera_object.rotation_euler = (math.radians(90 - (cam_z * 6)), 0, 0)
    
    photo_path = os.path.join(output_dir, f"new_data_photo_angle_{i}.png")
    bpy.context.scene.render.filepath = photo_path
    bpy.ops.render.render(write_still=True)
    print(f"Rendered photo {i}/{num_photos}: {photo_path}")

# --- 3. Render 360 Turntable Video (60 frames) ---
bpy.context.scene.render.image_settings.file_format = 'FFMPEG'
bpy.context.scene.render.ffmpeg.format = 'MPEG4'
bpy.context.scene.render.ffmpeg.codec = 'H264'
bpy.context.scene.render.ffmpeg.constant_rate_factor = 'HIGH'
bpy.context.scene.render.filepath = os.path.join(output_dir, "new_data_turntable.mp4")

# Set frame range
bpy.context.scene.frame_start = 1
bpy.context.scene.frame_end = 60
bpy.context.scene.render.fps = 30

# Animate model_parent Z rotation
model_parent.rotation_euler = (0, 0, 0)
model_parent.keyframe_insert(data_path="rotation_euler", frame=1)
model_parent.rotation_euler = (0, 0, math.radians(360))
model_parent.keyframe_insert(data_path="rotation_euler", frame=60)

# Set interpolation to linear for smooth continuous loop
for fcurve in model_parent.animation_data.action.fcurves:
    for kp in fcurve.keyframe_points:
        kp.interpolation = 'LINEAR'

# Render animation
bpy.ops.render.render(animation=True)
print("Turntable video render completed.")
