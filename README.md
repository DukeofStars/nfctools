# NebTools

An all-in-one toolkit for managing and editing Nebulous: Fleet Command fleets.

## Features
 - [x] Edit fleet descriptions
 - [x] Edit liner hulls and dressings
 - [x] Merge fleets together
 - [x] Support custom fleet directories
 - [x] Supports filtering fleets based on Unix shell-style patterns
 - [ ] Search through fleets
 - [ ] Edit fleet formations geometrically

## Liner editing
The available liner hull segments can be found here:

### Marauders
#### Bows
##### Bow A
Thin section, one mount angled top, one mount bottom, broadside mounts placed below modules
2 side thrusters on each side, 2 medium thrusters rear

##### Bow B
Thin section, one mount angled top, one mount bottom, broadside mounts placed above modules
2 thrusters angled top, 2 thrusters angled bottom side, 2 side thrusters on each side, 2 medium thrusters rear

##### Bow C
Large long wide and flat section, one mount top set far back, one mount bottom
2 thrusters top, 2 thrusters bottom, 2 side thrusters, 2 medium thrusters rear
!!!BRIDGES CANNOT BE MOUNTED ON THIS SECTION!!!
Setting the bridge section to ‘0’ will crash the fleet editor when it tries to load the fleet

#### Cores
##### Core A
Long, Thin section, two mounts angled top, one mount bottom, broadside mounts behind compartments

##### Core B
Long, Thin section, two mounts angled top, one mount bottom, broadside mounts ahead of compartments

##### Core C
Wide section, two mounts flat top, one mount bottom

#### Sterns
##### Stern A
Quad Nacelles, one mount on each side
Two main thrusters, 4 medium thrusters forward, 2 side thrusters on each side

##### Stern B
Short section triple thruster w/ double nacelles, one mount top, one mount bottom
3 main thrusters, 2 medium thrusters forward, 2 medium thrusters rear, 2 side thrusters on each side.
!!!BRIDGES CANNOT BE MOUNTED ON THIS REAR SECTION!!!
Setting the bridge section to ‘2’ will crash the fleet editor when it tries to load the fleet

##### Stern C
Large long quad engine, one mount top, one mount bottom
4 main thrusters, 2 side thrusters on each side.

### Moorlines
#### Bows
##### Bow A
Over/under container setup. Over/under PD set forward
!!!BRIDGES CANNOT BE MOUNTED ON THIS SECTION!!!
Setting the bridge section to ‘0’ will crash the fleet editor when it tries to load the ship

##### Bow B
Angled top container setup. Over under pd, top set forward, bottom set back

##### Bow C
Side by side container setup. Over under pd, top set forward, bottom set back

#### Cores
##### Core A
Over/under container setup. 2 PD side to side back, 1pd bottom forward
Longer compartment section

##### Core B
Angled top container setup. 1 PD top, 2 bottom, triangle pattern

##### Core C
Side by side container setup. 2 pd top, 1 pd bottom forward outset.

#### Sterns
##### Stern A
Pd top bottom

##### Stern B
Pd top bottom

##### Stern C
Pd side to side

### Superstructures
##### A
Large two deck bridge with outcroppings 

##### B
Large, blocky bridge

##### C
Bridge with mast

##### D
Tall thin bridge with outcropping


## Configuration
NebTools supports a couple of configuration options which can be set at `%APPDATA%/NebTools/config/config.toml` (or equivalent on other platforms. If you aren't sure, you can check the logs by running the app manually from a terminal). Currently, those are:
 - `saves_dir`: The path to the Nebulous saves directory. On windows this is usually at `C:\Program Files (x86)\Steam\steamapps\common\Nebulous\Saves`. Note that this does not point to the Fleets directory, but to it's parent.
 - `excluded_dirs`: A list of Unix shell-style patterns that will not be displayed in the app. e.g. If you like to keep your old fleets around but don't like them cluttering the app, you could set this to: `excluded_dirs = ["**/Old/**/*"]`. Or, if you don't want to show the starter fleets, something like this: `excluded_dirs = ["**/Starter Fleets - Alliance/*", "**/Starter Fleets - Protectorate/*"]`.
 If you aren't familiar with Unix shell-style patterns, here is a quick start. `**` means any subdirectory and it's subdirectories, `*` means any file within a directory. There is a lot more you can do with this however, for example matching different variations of a file or folder name.