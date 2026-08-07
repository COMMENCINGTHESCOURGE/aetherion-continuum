extends VehicleBody3D

@export var engine_force: float = 200.0
@export var steering_angle: float = 0.5

var front_wheels: Array[VehicleWheel3D] = []
var rear_wheels: Array[VehicleWheel3D] = []


func _ready() -> void:
	# Find front and rear wheels
	for child in get_children():
		if child is VehicleWheel3D:
			var wheel = child as VehicleWheel3D
			if wheel.use_as_steering:
				front_wheels.append(wheel)
			if wheel.use_as_traction:
				rear_wheels.append(wheel)


func _physics_process(_delta: float) -> void:
	var steer: float = 0.0
	var throttle: float = 0.0

	if Input.is_action_pressed("ui_right"):
		steer -= steering_angle
	if Input.is_action_pressed("ui_left"):
		steer += steering_angle
	if Input.is_action_pressed("ui_up"):
		throttle += engine_force
	if Input.is_action_pressed("ui_down"):
		throttle -= engine_force / 2.0  # reverse

	# Apply steering to front wheels
	for wheel in front_wheels:
		wheel.steering = steer

	# Apply engine force
	engine_force = throttle
