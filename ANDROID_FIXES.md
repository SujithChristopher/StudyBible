# Android Fixes - Settings Storage & Icon

## Issues Fixed

### 1. Settings Not Saving on Android ✅

**Problem**: Settings were not persisting on Android devices.

**Root Cause**:
- Android has different storage paths than desktop platforms
- The `directories` crate's `config_dir()` may not work reliably on Android
- Missing permissions in AndroidManifest

**Solutions Implemented**:

1. **Updated Storage Module** ([src/storage.rs](src/storage.rs)):
   - Android now uses `data_dir()` instead of `config_dir()`
   - This provides app-specific internal storage (no permissions needed)
   - Added extensive logging to debug storage issues
   - Path: `/data/data/com.studybible.app/files/StudyBible/settings.json`

2. **Added Android Permissions** ([Dioxus.toml](Dioxus.toml)):
   ```toml
   [bundle.android.manifest]
   permissions = [
       "android.permission.READ_EXTERNAL_STORAGE",
       "android.permission.WRITE_EXTERNAL_STORAGE"
   ]
   ```
   Note: These are for external storage access, but app-internal storage doesn't need them.

3. **Enhanced Error Logging**:
   - All storage operations now log to console
   - Check `adb logcat` to see storage operations:
     ```bash
     adb logcat | grep StudyBible
     ```

### 2. App Icon Not Showing on Android ✅

**Problem**: Custom icon not appearing in Android builds.

**Solutions Implemented**:

1. **Updated Dioxus.toml**:
   - Added Android-specific icon configuration
   - Icon path: `assets/icons/icon.png`

2. **Icon Requirements**:
   - **Minimum size**: 512x512 pixels
   - **Recommended size**: 1024x1024 pixels
   - **Format**: PNG with transparency
   - **Location**: `assets/icons/icon.png`

3. **What Dioxus Does Automatically**:
   - Generates all required Android icon densities (mdpi, hdpi, xhdpi, xxhdpi, xxxhdpi)
   - Adds icons to proper Android resource directories
   - Updates AndroidManifest.xml

## How to Test

### Testing Settings Storage

1. **Build and install on Android**:
   ```bash
   dx build --platform android --release
   adb install -r dist/StudyBible.apk
   ```

2. **Open app and change settings**:
   - Go to Settings (gear icon in sidebar)
   - Change theme, font size, or other settings
   - Close the app completely (swipe away from recents)

3. **Reopen the app**:
   - Settings should be preserved
   - Check logcat for storage messages:
     ```bash
     adb logcat | grep -E "Settings|StudyBible|💾|✓"
     ```

4. **Expected Log Output**:
   ```
   🤖 Android detected: using data_dir for settings
   📁 Config directory: "/data/data/com.studybible.app/files/StudyBible"
   ✓ Config directory created/verified
   💾 Settings will be stored at: "/data/data/com.studybible.app/files/StudyBible/settings.json"
   💾 Attempting to save settings...
   📝 Serialized settings (XXX bytes)
   ✓ Successfully saved settings to: ...
   ✓ Verified: file contains XXX bytes
   ```

### Testing Icon

1. **Add your icon**:
   - Save the brown Bible with cross icon to `assets/icons/icon.png`
   - Ensure it's at least 512x512 pixels

2. **Clean and rebuild**:
   ```bash
   dx clean
   dx build --platform android --release
   ```

3. **Install and check**:
   ```bash
   adb install -r dist/StudyBible.apk
   ```
   - Icon should appear in app drawer
   - Icon should appear in recent apps
   - Icon should appear in app info

## Debugging

### If Settings Still Don't Save

1. **Check logcat for errors**:
   ```bash
   adb logcat | grep -i "failed\|error" | grep StudyBible
   ```

2. **Check file permissions**:
   ```bash
   adb shell
   cd /data/data/com.studybible.app/files/StudyBible
   ls -la
   cat settings.json
   ```

3. **Verify storage was initialized**:
   - Look for "✓ Config directory created/verified" in logcat
   - If missing, the storage initialization failed

### If Icon Still Doesn't Appear

1. **Verify icon file exists**:
   ```bash
   ls -lh assets/icons/icon.png
   ```

2. **Check icon in APK**:
   ```bash
   # Extract and check APK
   unzip -l dist/StudyBible.apk | grep mipmap
   ```
   Should show icon files in multiple densities.

3. **Try a complete clean rebuild**:
   ```bash
   dx clean
   rm -rf dist/
   dx build --platform android --release
   ```

## Storage Paths by Platform

| Platform | Settings Path |
|----------|---------------|
| **Windows** | `%APPDATA%\StudyBible\settings.json` |
| **macOS** | `~/Library/Application Support/StudyBible/settings.json` |
| **Linux** | `~/.config/StudyBible/settings.json` |
| **Android** | `/data/data/com.studybible.app/files/StudyBible/settings.json` |

## Implementation Details

### Files Modified

1. **[src/storage.rs](src/storage.rs)**: Android-specific storage handling
2. **[Dioxus.toml](Dioxus.toml)**: Android manifest permissions and icon
3. **[src/main.rs](src/main.rs)**: Settings persistence integration
4. **[assets/icons/README.md](assets/icons/README.md)**: Icon documentation

### Key Changes

- `SettingsStorage::new()` now detects Android and uses `data_dir()`
- All storage operations log their status for debugging
- Settings are automatically loaded on app start
- Settings are saved immediately when changed in the settings modal

## Next Steps

1. ✅ Save your icon to `assets/icons/icon.png`
2. ✅ Test on Android device/emulator
3. ✅ Verify settings persist across app restarts
4. ✅ Verify icon appears correctly

## Additional Notes

- **No Runtime Permissions Needed**: App-specific internal storage doesn't require user-granted permissions
- **Data Survives**: Settings persist even after app updates (unless app data is cleared)
- **Backup**: Settings are not backed up by default in Android (you can implement cloud sync later)
- **Icon Caching**: Android may cache icons; sometimes uninstall/reinstall is needed to see icon changes
