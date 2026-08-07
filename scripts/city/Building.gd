extends Node3D

@export var building_mesh: Mesh = null
@export var zone_type: Zone.ZoneType = Zone.ZoneType.RESIDENTIAL
@export var floors: int = 1

var occupancy: float = 0.0
var jobs_provided: int = 0


func _ready() -> void:
	setup_building()


func setup_building() -> void:
	# Create visual representation if not provided
	if not $MeshInstance3D:
		var mesh_instance = MeshInstance3D.new()
		mesh_instance.name = "MeshInstance3D"
		
		var box_mesh = BoxMesh.new()
		var base_size = Vector3(2, 4, 2)
		box_mesh.size = base_size * floors
		mesh_instance.mesh = box_mesh
		
		# Color based on zone type
		var material = StandardMaterial3D.new()
		match zone_type:
			Zone.ZoneType.RESIDENTIAL:
				material.albedo_color = Color(0.2, 0.6, 0.2)  # Green
			Zone.ZoneType.COMMERCIAL:
				material.albedo_color = Color(0.2, 0.2, 0.8)  # Blue
			Zone.ZoneType.INDUSTRIAL:
				material.albedo_color = Color(0.8, 0.6, 0.2)  # Orange
		
		mesh_instance.material_override = material
		add_child(mesh_instance)
	
	# Calculate stats based on floors and zone type
	calculate_stats()


func calculate_stats() -> void:
	match zone_type:
		Zone.ZoneType.RESIDENTIAL:
			occupancy = floors * 10.0
		Zone.ZoneType.COMMERCIAL:
			jobs_provided = floors * 5
		Zone.ZoneType.INDUSTRIAL:
			jobs_provided = floors * 8


func upgrade() -> void:
	floors += 1
	setup_building()
