extends Node

var ship
# Declare member variables here. Examples:
# var a = 2
# var b = "text"
var entity_node

# Called when the node enters the scene tree for the first time.
func _ready():
	
	pass # Replace with function body.

func process_ship(player, world):
	var entities = player.get_parent()
	for i in entities.get_children():
		if i.actor_type == "table":
			if Input.is_key_pressed(KEY_SHIFT):
				i.translation.y += 0.05
				i.packet_cooldown = 0
				i._network_share()
			elif Input.is_key_pressed(KEY_CONTROL):
				i.translation.y -= 0.05
				i.packet_cooldown = 0
				i._network_share()


func get_prop_by_ref(ref):
	for obj in PlayerData.inventory:
		if obj.ref == ref:
			return obj
	return null

func is_sitting_on_ship(player):

	for prop in PlayerData.props_placed:
		var obj = get_prop_by_ref(prop.ref)
		if obj != null && obj.id == 'prop_table':
			ship = obj
			return player.sitting
	return false;
	
