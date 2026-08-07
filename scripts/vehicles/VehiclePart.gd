extends Resource
class_name VehiclePart

@export var part_name: String = "Unnamed Part"
@export var mesh: PackedScene          # 3D model scene or Mesh resource
@export var mass: float = 10.0
@export var center_of_mass: Vector3 = Vector3.ZERO
@export var drag_coefficient: float = 0.3

# Attachment points: name -> transform (relative to part)
@export var attachment_points: Dictionary = {}

# Material properties – affects durability, weight, cost, etc.
@export var material_type: String = "Steel"
@export var durability: float = 100.0
@export var cost: int = 50


func get_attachment_point(point_name: String) -> Transform3D:
	return attachment_points.get(point_name, Transform3D.IDENTITY)
