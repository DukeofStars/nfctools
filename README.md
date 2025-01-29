# NebTools

An all-in-one toolkit for managing and editing Nebulous: Fleet Command fleets.

## Contents
 - Features
 - Known bugs
 - Installation
 - How to...
 - Configuration
 - Liner editing

## Features
 - [x] Edit fleet descriptions
 - [x] Edit liner hulls and dressings (the app will stop you from creating a broken liner layout, no need to worry about putting a bridge on the wrong segment and crashing the game)
 - [x] Merge fleets together
 - [x] Supports custom saves directories (with automatic detection)
 - [x] Supports filtering fleets based on Unix shell-style patterns
 - [x] Supports tagging fleets, which are visible in game with colours (Searching by tag is something I hope to add in the future)
 - [x] Apply updated missile templates to multiple fleets at once
 - [x] Integrated win predictor
 - [ ] Saving and re-using liner hull config templates
 - [ ] Edit fleet formations geometrically
 - [ ] Search through fleets

## Known bugs
 - Dressing descriptions don't match moorline hull segments.
 - Selecting a fleet with 0 ships crashes the program.

## Installation
### Via installer (Windows)
 - Go to the latest release
 - Download `nebtools-setup.exe`
 - Launch the setup and follow instructions

### Manually (Windows)
 - Go to the latest release
 - Download `nfctools.exe`
 - Save this exe somewhere
 - Launch it manually whenever you wish to

### Other OS
**Note**: If you are using an operating system other than windows it is assumed you will have decent technical knowledge.

To install on another operating system, you will need to install the latest Rust toolchain [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install). You will also need the files of this repository. You can either download as a zip (Click the green 'Code' button on GitHub), or clone it using git.
Now, navigate to the downloaded repository root in a terminal (There should be a `Cargo.toml` and a `src/`, as well as other files and folders) and run `cargo build --release`. The output binary should be located at `target/release/nfctools` or similar.

## How to...
### Edit fleet descriptions
In the main window, you can select a fleet by clicking on it in the fleet list on the left, and then edit it's description using the text box on the right panel under 'Edit Description'. It will automatically save for you.

### Merge fleets together
Select two fleets by holding control and clicking them in the fleet list. Then, input a name in the textbox next to the 'Merge' button on the left panel, then click the button. The created fleet will be placed in your root Fleets directory.

### Edit liner hulls and dressings
Select the fleet which contains the ships you wish to edit, then click 'Open Fleet Editor' on the right panel at the top. In the new window, select the ship you want to edit. If it's a marauder or moorline, settings will open up next to the list where you can edit the different segment types, where the bridge is located, and what dressings for each segment. All of the hull segments and bridge types can be found at the bottom of this document.

### Tag fleets
Just above the edit description textbox, there is a tag creation menu, where you can give it a name and a custom RGB colour. When you add a tag, the app remembers it's colour, and the next time you type in that tag name it will automatically fill in the colour. These tags are visible in game just above the description with their custom colours (in fact, the current implementation simply injects the tags at the start of the description). You can remove tags by clicking on them in the grid. **NOTE: The ui currently only supports up to 12 tags.**

### Apply missile templates to multiple fleets at once
At the top of the main window, click Missiles. In the new window, select the missile you wish to apply and click Update Fleets. A window will show up with a list of all the fleets that missile is in as well as a checkbox next to each one so you can customise which fleets to apply the new missile to. After you have picked your fleets, click Confirm.

### Predict victories based on points
At the top of the main window, click Win Predictor. Enter all appropriate values into the inputs, and the prediction will live update.

## Configuration
NebTools supports a couple of configuration options which can be set at `%APPDATA%/NebTools/config/config.toml` (or equivalent on other platforms. If you aren't sure, you can check the logs by running the app manually from a terminal). Currently, those are:
 - `saves_dir`: The path to the Nebulous saves directory. On windows this is usually at `C:\Program Files (x86)\Steam\steamapps\common\Nebulous\Saves`. Note that this does not point to the Fleets directory, but to it's parent. There shouldn't ever be a reason to set this unless your system is really wacky, the app should be able to detect the nebulous saves directory universally (if it doesn't, please let me know as well as providing information on your setup).
 - `excluded_dirs`: A list of Unix shell-style patterns that will not be displayed in the app. e.g. If you like to keep your old fleets around but don't like them cluttering the app, you could set this to: `excluded_dirs = ["**/Old/**/*"]`. Or, if you don't want to show the starter fleets, something like this: `excluded_dirs = ["**/Starter Fleets - Alliance/*", "**/Starter Fleets - Protectorate/*"]`.
 If you aren't familiar with Unix shell-style patterns, here is a quick start. `**` means any subdirectory and it's subdirectories, `*` means any file within a directory. There is a lot more you can do with this however, for example matching different variations of a file or folder name.

## Liner editing
The available liner hull segments can be found here:

### Marauders
#### Bows
 - **A**: Thin section, one mount angled top, one mount bottom, broadside mounts placed below modules
2 side thrusters on each side, 2 medium thrusters rear

 - **B**: Thin section, one mount angled top, one mount bottom, broadside mounts placed above modules
2 thrusters angled top, 2 thrusters angled bottom side, 2 side thrusters on each side, 2 medium thrusters rear

 - **C**: Large long wide and flat section, one mount top set far back, one mount bottom
2 thrusters top, 2 thrusters bottom, 2 side thrusters, 2 medium thrusters rear
!!!BRIDGES CANNOT BE MOUNTED ON THIS SECTION!!!
Setting the bridge section to ‘0’ will crash the fleet editor when it tries to load the fleet

#### Cores
 - **A**: Long, Thin section, two mounts angled top, one mount bottom, broadside mounts behind compartments

 - **B**: Long, Thin section, two mounts angled top, one mount bottom, broadside mounts ahead of compartments

 - **C**: Wide section, two mounts flat top, one mount bottom

#### Sterns
 - **A**: Quad Nacelles, one mount on each side
Two main thrusters, 4 medium thrusters forward, 2 side thrusters on each side

 - **B**: Short section triple thruster w/ double nacelles, one mount top, one mount bottom
3 main thrusters, 2 medium thrusters forward, 2 medium thrusters rear, 2 side thrusters on each side.
!!!BRIDGES CANNOT BE MOUNTED ON THIS REAR SECTION!!!
Setting the bridge section to ‘2’ will crash the fleet editor when it tries to load the fleet

 - **C**: Large long quad engine, one mount top, one mount bottom
4 main thrusters, 2 side thrusters on each side.

### Moorlines
#### Bows
 - **A**: Over/under container setup. Over/under PD set forward
!!!BRIDGES CANNOT BE MOUNTED ON THIS SECTION!!!
Setting the bridge section to ‘0’ will crash the fleet editor when it tries to load the ship

 - **B**: Angled top container setup. Over under pd, top set forward, bottom set back

 - **C**: Side by side container setup. Over under pd, top set forward, bottom set back

#### Cores
 - **A**: Over/under container setup. 2 PD side to side back, 1pd bottom forward
Longer compartment section

 - **B**: Angled top container setup. 1 PD top, 2 bottom, triangle pattern

 - **C**: Side by side container setup. 2 pd top, 1 pd bottom forward outset.

#### Sterns
 - **A**: Pd top bottom

 - **B**: Pd top bottom

 - **C**: Pd side to side

### Superstructures
 - **A**: Large two deck bridge with outcroppings 

 - **B**: Large, blocky bridge

 - **C**: Bridge with mast

 - **D**: Tall thin bridge with outcropping