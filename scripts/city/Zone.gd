extends Area3D
class_name Zone

enum ZoneType { RESIDENTIAL, COMMERCIAL, INDUSTRIAL }

@export var zone_type: ZoneType = ZoneType.RESIDENTIAL
@export var density: int = 1           # low, medium, high
@export var max_buildings: int = 10
@export var growth_time: float = 30.0  # seconds to grow

var buildings: Array[Node] = []
var timer: float = 0.0


func _ready() -> void:
	# Connect to area signals if needed
	pass


func _process(delta: float) -> void:
	timer += delta
	if timer >= growth_time and buildings.size() < max_buildings:
		try_grow()
		timer = 0.0


func try_grow() -> void:
	# Check conditions (e.g., enough resources, demand)
	if can_grow():
		var building = spawn_building()
		buildings.append(building)


func can_grow() -> bool:
	# Placeholder: always true for prototype
	return true


func spawn_building() -> Node:
	# Instantiate a building scene based on zone type
	var building_scene = load("res://scenes/city/Building.tscn")
	if building_scene == null:
		# Fallback: create a simple mesh if scene doesn't exist yet
		var mesh_instance = MeshInstance3D.new()
		var box_mesh = BoxMesh.new()
		box_mesh.size = Vector3(2, 4, 2)
		mesh_instance.mesh = box_mesh
		mesh_instance.position = get_random_position_within_zone()
		add_child(mesh_instance)
		return mesh_instance
	
	var instance = building_scene.instantiate()
	instance.position = get_random_position_within_zone()
	add_child(instance)
	return instance


func get_random_position_within_zone() -> Vector3:
	# Simple random offset within the area (collision shape bounds)
	var collision_shape = $CollisionShape3D as CollisionShape3D
	if collision_shape and collision_shape.shape is BoxShape3D:
		var shape = collision_shape.shape as BoxShape3D
		var extents = shape.size / 2
		return Vector3(
			randf_range(-extents.x, extents.x),
			extents.y,  # Place on top of the zone
			randf_range(-extents.z, extents.z)
		)
	return Vector3.ZERO
