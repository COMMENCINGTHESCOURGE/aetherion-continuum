extends Node

var zones: Array[Zone] = []


func register_zone(zone: Zone) -> void:
	zones.append(zone)


func get_zones_of_type(type: Zone.ZoneType) -> Array[Zone]:
	var result: Array[Zone] = []
	for z in zones:
		if z.zone_type == type:
			result.append(z)
	return result


# Called from UI or other systems
func zone_area(area_node: Node, zone_type: Zone.ZoneType) -> void:
	# Convert selected area into a Zone
	if area_node is Area3D:
		if not area_node.has_script():
			area_node.set_script(load("res://scripts/city/Zone.gd"))
		var zone = area_node as Zone
		zone.zone_type = zone_type
		register_zone(zone)
