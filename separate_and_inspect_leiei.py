import bpy
import os

gltf_path = r"C:\Users\dasha\Projects\aetherion-continuum\leiei.gltf"

bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.import_scene.gltf(filepath=gltf_path)

print("\n=== IMPORTED SCENE OBJECTS ===")
with open(r"C:\Users\dasha\Projects\aetherion-continuum\inspect_results.txt", "w", encoding="utf-8") as out:
    out.write("=== IMPORTED SCENE OBJECTS ===\n")
    for obj in bpy.context.scene.objects:
        out.write(f"Object: {obj.name}, Type: {obj.type}\n")

    # Separate loose parts of mesh objects
    for obj in list(bpy.context.scene.objects):
        if obj.type == 'MESH':
            out.write(f"\nSeparating loose parts for: {obj.name}...\n")
            bpy.context.view_layer.objects.active = obj
            obj.select_set(True)
            bpy.ops.object.mode_set(mode='EDIT')
            bpy.ops.mesh.separate(type='LOOSE')
            bpy.ops.object.mode_set(mode='OBJECT')

    out.write("\n=== SEPARATED PARTS LIST ===\n")
    for obj in sorted(bpy.context.scene.objects, key=lambda x: x.name):
        if obj.type == 'MESH':
            bbox = obj.bound_box
            center_x = sum([v[0] for v in bbox]) / 8.0
            center_y = sum([v[1] for v in bbox]) / 8.0
            center_z = sum([v[2] for v in bbox]) / 8.0
            out.write(f"Name: {obj.name:<25} Verts: {len(obj.data.vertices):<5} Center: ({center_x:.3f}, {center_y:.3f}, {center_z:.3f})\n")
