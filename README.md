
<p align="center"><img width="1920" height="1080" alt="elementipelago" src="https://github.com/user-attachments/assets/eca27809-dd3b-4142-a6d9-ea6cafc69802" /></p>
<p align="center">
  <img alt="License" src="https://img.shields.io/github/license/peppidesu/elementipelago?style=for-the-badge"></img>
  <img alt="GitHub Release" src="https://img.shields.io/github/v/release/peppidesu/elementipelago?style=for-the-badge&color=blue">
  <img alt="GitHub branch check runs" src="https://img.shields.io/github/check-runs/peppidesu/elementipelago/main?style=for-the-badge">
  <a href="https://archipelago.miraheze.org/wiki/Elementipelago"><img alt="Static Badge" src="https://img.shields.io/badge/Archipelago-Wiki-red?style=for-the-badge"></object></a>
</p>
<h4 align="center">
  Elementipelago is an alchemy-game developed for <a href="https://archipelago.gg">Archipelago</a>. Think doodle-god, but now for your favorite multiworld randomizer.
</h4>

## Setup

1. Download and install the `elementipelago.apworld` from the latest release.
2. Generate your YAML using the Options Creator in the Archipelago Launcher and generate your multiworld.
3. Host the AP server locally or on a remote that accepts SSL connections (like `archipelago.gg`).
4. Go to https://elementipelago.peppidesu.dev/ and connect to your slot.

> [!NOTE]
> If you are unable to setup SSL on the remote you are using, you will need to spin up a local version of the web client and connect to it using non-secure HTTP.

### Older AP-world versions
> [!CAUTION]
> We no longer support AP-worlds older than v1.0.0-rc1. Consider slow release or regenerating if this version is used in your multiworld.

Starting from v1.0.0, the official web client will support both the current and previous major versions of the AP-world. This should give players a decent time window to finish existing games and update to the latest version.

## How to play
In Elementipelago, the entire crafting tree is randomly generated, so every generated game is completely different!

The items you receive are **Elements** used for crafting. You combine elements from the drawer to create products. Products can be one of the following:

- **Intermediates**: Don't give checks, but can be used for further crafting.
- **Compounds**: Give checks, but aren't used for crafting (unless `compounds_are_ingredients` is enabled).

You can also receive the following upgrades:
- **Progressive Filter**: Enables marking and filtering items that are:
  - exhausted (can no longer be combined to form new products); and
  - BK (cannot currently be combined to form new products).
- **Progressive Item Limit**: Increases the maximum number of items that can be placed at the same time (starts at 10).

### Naming
Internally and in other clients, all items appear as "Item-Type XYZ" (e.g. "Compound 23"). These are also visible in the drawer and when hovering over an item. To make the gameplay more interesting though, Elementipelago generates a display name for each item:
- Elements that originate from other worlds appear as the location where they
are found in-game.
- Intermediates and starting elements get a randomly generated name.

### Hints
To make received hints useful to the player, their recipes are shown in the "Hints" panel found in the top right tray. In case you are really stuck though, one can use `/explain` in [Universal Tracker](https://github.com/FarisTheAncient/Archipelago/releases?q=Tracker) to reveal the entire recipe tree for a given item.

<p align="center"><img src="https://github.com/user-attachments/assets/de9941ec-5274-44a8-b703-898e618a9bb2" width="640"/></p>

## License

The Elementipelago source code is licensed under the AGPL-3.0 license provided in the LICENSE file.

All sprites are © 2026 by Pepijn Bakker & Noa Aarts and licensed under CC BY-NC 4.0. To view a copy of this license, visit https://creativecommons.org/licenses/by-nc/4.0/

SFX include samples sourced from freesound.com. Required attributions are listed below:

- Doorbell Pull with pull Store Bell 05.wav by maisonsonique -- https://freesound.org/s/196372/ -- License: Attribution 4.0
- Door, Front, Opening, A.wav by InspectorJ -- https://freesound.org/s/431117/ -- License: Attribution 4.0
