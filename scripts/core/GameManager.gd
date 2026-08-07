extends Node

# Global game state
var current_money: int = 10000
var population: int = 0
var total_vehicles: int = 0

signal money_changed(new_amount: int)
signal population_changed(new_pop: int)


func _ready() -> void:
	print("Game initialized!")


func add_money(amount: int) -> void:
	current_money += amount
	money_changed.emit(current_money)


func spend_money(amount: int) -> bool:
	if current_money >= amount:
		current_money -= amount
		money_changed.emit(current_money)
		return true
	return false


func register_vehicle() -> void:
	total_vehicles += 1


func update_population(delta: int) -> void:
	population += delta
	population_changed.emit(population)
