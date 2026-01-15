Small set of quality-of-life addons for the PF2e game system in [Foundry VTT](https://foundryvtt.com/).

## Current Features

* Damage Popout
    * Automatically open popup when an actor you control is prompted to take damage or make a save.
* Visible Equipment Preview
    * Allow players to see the icons for items that are worn or held by NPCs & monsters they wouldn't normally have visibility into via a macro.
        ```game.modules.get("johnys-module").api.openEquipmentScreen()```
    * Optionally integrates with [PF2e Bestiary Tracking](https://github.com/WBHarry/pf2e-bestiary-tracking) to show visible equipment on the player's limited bestiary view.
* Written in rust 🦀


## Supported and Recommended Modules

* [PF2e Bestiary Tracking](https://github.com/WBHarry/pf2e-bestiary-tracking)
* [Dice so Nice](https://gitlab.com/riccisi/foundryvtt-dice-so-nice)
* [PF2e Toolbelt](https://github.com/reonZ/pf2e-toolbelt)

## Foundry Install Manifest URL

```
https://github.com/johnyburd/johnys-pf2e-qol/releases/latest/download/module.json
```
