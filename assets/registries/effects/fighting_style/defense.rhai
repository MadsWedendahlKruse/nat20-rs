fn armor_class_hook(entity_view) {
    // Only apply if the entity is wearing armor
    let armor_type = entity_view.loadout.armor_type;
    if armor_type != "None" && armor_type != "Clothing" {
        // Grant +1 bonus to AC
        return 1;
    }
    
    return 0;
}