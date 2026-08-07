# City Vehicle Builder - Godot 4 Prototype

A prototype for a city-building and vehicle construction simulation game.

## 🎮 Features

### Vehicle System
- Modular vehicle parts (Chassis, Wheels, Engine) defined as Resources
- Jolt Physics integration for realistic vehicle dynamics
- Drive with Arrow Keys (Up/Down for throttle, Left/Right for steering)

### City Zoning System
- Three zone types: Residential (Green), Commercial (Blue), Industrial (Orange)
- Automatic building growth over time
- ZoneManager singleton for global zone tracking

## 📁 Project Structure

```
project_root/
├── assets/              # 3D models, textures, audio
├── scripts/
│   ├── core/           # GameManager, ZoneManager (autoloads)
│   ├── vehicles/       # VehiclePart, Chassis, Wheel, Engine, VehicleController
│   ├── city/           # Zone, Building
│   └── ui/             # UI controllers
├── scenes/             # .tscn scene files
├── resources/          # Pre-made .tres resource definitions
└── project.godot       # Godot project configuration
```

## 🚀 Getting Started

1. **Open in Godot**: Import this project in Godot 4.2+
2. **Run the Main Scene**: Press F5 to run `scenes/main.tscn`
3. **Test Driving**: Use Arrow Keys to drive the vehicle
4. **Watch Buildings**: Observe buildings spawning in the Zone area

## 🎯 Next Steps

- [ ] Create custom 3D models for vehicle parts
- [ ] Implement vehicle assembly UI
- [ ] Add more building types and upgrade mechanics
- [ ] Create resource (.tres) definitions for parts
- [ ] Add economy and citizen simulation
- [ ] Implement save/load system

## 🛠️ Development

### Adding New Vehicle Parts

1. Create a new script extending `VehiclePart`
2. Define custom properties
3. Create `.tres` resource files in `resources/parts/`

### Creating New Zones

1. Duplicate existing Zone node
2. Set `zone_type` export variable
3. Register with ZoneManager

## 📄 License

MIT License - Feel free to use and modify for your projects!
