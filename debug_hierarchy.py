import bpy

gltf_path = r"C:\Users\dasha\Downloads\new data.gltf"

bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.import_scene.gltf(filepath=gltf_path)

print("--- Object Hierarchy ---")
for obj in bpy.context.scene.objects:
    print(f"Name: {obj.name}, Type: {obj.type}, Location: {obj.location}, Parent: {obj.parent.name if obj.parent else 'None'}")
    if obj.type == 'MESH':
        print(f"  Mesh vertices: {len(obj.data.vertices)}")
