# BluetoothTimeout
Application to automatically disconnect a Bluetooth device after a timeout.

## Features
- Safe & helpful wrapper over Win32 bluetooth api
- App to connect to devices and automatically disconnect after timeout

## Usage
Either download a compiled executable from [releases](../../releases/) or build from source using `cargo build --release` (rust required).
Once you launch the executable, ensure bluetooth is turned on, then scan for devices.
I'm not sure why, but you may need to first open the bluetooth widget in the taskbar before the app can discover devices.
Once connected, Windows wil prompt you to connect, and once you do so, you will be able to set a timeout for the device. Timeouts can be set like `1h 25s`.

## Motivation
Usually, after connecting my bluetooth speaker to my PC, I'll forget about it until the next day when I try to play something and sounds starts blasting out of it from across the room.
This is a simple utility app that lets you connect to a bluetooth device and set a timeout. Once the timeout expires, the bluetooth device will be automatically disconnected.
If you close the program, all devices connected through it are also disconnected.
Since no bluetooth libraries handled classic bluetooth on Windows, I had to make my own wrapper for the Win32 api using the windows-rs crate.
Currently, when connecting the authentication is handled through Windows settings' popup, but I might handle it in the app at a future date.
I made this app primarily for myself, but if you'd like to see a change, open a PR or create an issue!
