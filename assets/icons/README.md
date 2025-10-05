# App Icons

## Required Icon Sizes

Place your icon image here with the name `icon.png` (minimum 512x512 pixels, preferably 1024x1024).

### Android Icon Requirements

For Android builds, Dioxus will automatically generate the following icon sizes:
- **mdpi**: 48x48 px
- **hdpi**: 72x72 px
- **xhdpi**: 96x96 px
- **xxhdpi**: 144x144 px
- **xxxhdpi**: 192x192 px

### Desktop Icon Requirements

For desktop platforms (Windows, macOS, Linux):
- The icon will be automatically resized from `icon.png`

## Current Setup

1. Save your brown Bible with cross icon as `icon.png` in this directory
2. Recommended size: **1024x1024 pixels**
3. Format: PNG with transparency
4. The icon will be automatically processed during build

## How to Add Your Icon

1. Save the icon image you have as `assets/icons/icon.png`
2. Rebuild the app:
   ```bash
   # For desktop
   dx build --platform desktop

   # For Android
   dx build --platform android
   ```

## Troubleshooting Android Icons

If the icon doesn't appear in Android builds:

1. **Clean the build**:
   ```bash
   dx clean
   ```

2. **Rebuild completely**:
   ```bash
   dx build --platform android --release
   ```

3. **Check logcat** for any icon-related errors:
   ```bash
   adb logcat | grep StudyBible
   ```

4. **Verify the icon file**:
   - Must be in PNG format
   - Should be at least 512x512 pixels
   - Path must be `assets/icons/icon.png`
