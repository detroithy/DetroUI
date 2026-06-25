# DetroUI - 8 High Priority Fixes

## #1 Shortcut Position Reset
- commands.rs:945-954: Grid-based position (col=count%6, row=count/6, x=40+col*90, y=40+row*100)

## #3 Music Player Volume Control
- commands.rs: New commands media_volume_up/down/mute using SendKeys
- tauri-bridge.js: mediaVolumeUp/Down/Mute
- music-player/index.html: Add volume buttons

## #5 WiFi/Volume Tray Buttons
- taskbar/renderer.js: Add click handlers for networkIcon/volumeIcon

## #6 Explorer/Terminal/Browser Launch
- commands.rs:243-267: Fix launch_app path handling

## #7 Desktop Tabs Clickable
- desktop/index.html: Fix mousedown/click handlers for #sw

## #8 Search Bar Reopens
- taskbar/renderer.js:77-78: focus->click event

## #10 Shortcut Position Persistence
- Covered by #1 fix

## #11 Desktop Delete/Rename
- commands.rs: delete_desktop, rename_desktop
- virtual_desktop.rs: delete/rename methods
- tauri-bridge.js: deleteDesktop, renameDesktop
- lib.rs: Register 2 new commands
- desktop/index.html: Context menu + bottom bar right-click
