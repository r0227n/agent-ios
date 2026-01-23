---
name: device-setup
description: Setup mobile device environment for testing with agent-mobile CLI
user-invocable: true
allowed-tools: Bash
argument-hint: "[ios|android]"
---

# Device Setup Skill

Setup mobile testing environment with: `/device-setup $ARGUMENTS`

## Arguments

- `ios` - Setup iOS Simulator environment
- `android` - Setup Android Emulator environment

## iOS Setup

The iOS setup performs:

1. **Check Xcode installation**
   - Verify `xcrun simctl` is available

2. **Check idb_companion**
   - Verify idb_companion is installed and accessible
   - Check for running companions

3. **Boot Simulator**
   - List available simulators
   - Boot if none running

4. **Verify Connection**
   - Confirm agent-mobile can communicate with device

### Helper Script

```bash
{baseDir}/scripts/setup-ios.sh
```

## Android Setup

The Android setup performs:

1. **Check Android SDK**
   - Verify `adb` is available
   - Verify `emulator` command is available

2. **Check adb Server**
   - Start adb server if not running

3. **Boot Emulator**
   - List available AVDs
   - Start emulator if none running

4. **Verify Connection**
   - Wait for device to boot completely
   - Confirm agent-mobile can communicate

### Helper Script

```bash
{baseDir}/scripts/setup-android.sh
```

## Execution Flow

If `$ARGUMENTS` is `ios`, run:
```bash
{baseDir}/scripts/setup-ios.sh
```

If `$ARGUMENTS` is `android`, run:
```bash
{baseDir}/scripts/setup-android.sh
```

If no argument or unrecognized argument, show usage and available options.

## Example Usage

```bash
# Setup iOS environment
/device-setup ios

# Setup Android environment
/device-setup android
```

## Expected Output

### iOS

```
=== iOS Device Setup ===

>>> Checking Xcode installation...
  [OK] xcrun simctl available

>>> Checking idb_companion...
  [OK] idb_companion found at /usr/local/bin/idb_companion

>>> Checking simulator status...
  [INFO] Found simulator: iPhone 15 (xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx) - Shutdown

>>> Booting simulator...
  [OK] Simulator booted successfully

>>> Verifying connection...
  [OK] agent-mobile can communicate with device

=== Setup Complete ===
Device: iPhone 15
UDID: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
Status: Ready for testing
```

### Android

```
=== Android Device Setup ===

>>> Checking Android SDK...
  [OK] adb found at /path/to/android-sdk/platform-tools/adb
  [OK] emulator found at /path/to/android-sdk/emulator/emulator

>>> Checking adb server...
  [OK] adb server running

>>> Checking emulator status...
  [INFO] Available AVDs: Pixel_6_API_33, Pixel_7_API_34

>>> Booting emulator...
  [INFO] Starting Pixel_6_API_33...
  [OK] Emulator booted successfully

>>> Verifying connection...
  [OK] agent-mobile can communicate with device

=== Setup Complete ===
Device: emulator-5554
Status: Ready for testing
```

## Troubleshooting

### iOS

- **xcrun not found**: Install Xcode Command Line Tools
  ```bash
  xcode-select --install
  ```

- **idb_companion not found**: Install idb
  ```bash
  brew install idb-companion
  ```

- **Simulator won't boot**: Reset simulator
  ```bash
  xcrun simctl erase all
  ```

### Android

- **adb not found**: Add Android SDK to PATH
  ```bash
  export ANDROID_HOME=$HOME/Library/Android/sdk
  export PATH=$PATH:$ANDROID_HOME/platform-tools
  ```

- **No AVDs available**: Create an AVD using Android Studio or avdmanager

- **Emulator won't start**: Check HAXM/KVM installation for hardware acceleration
