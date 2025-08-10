import os
import shutil
import subprocess


def run_command(command):
    """Run a shell command and exit on failure."""
    try:
        subprocess.run(command, check=True, stderr=subprocess.STDOUT)
    except subprocess.CalledProcessError as e:
        print(f"Command failed: {' '.join(command)} \nerror: {e}")
        exit(1)


def sign_directory(directory, signing_cert):
    """Recursively sign all executable files in the directory."""
    for root, dirs, files in os.walk(directory):
        for file in files:
            filepath = os.path.join(root, file)
            if os.access(filepath, os.X_OK):  # Check if file is executable
                run_command(['codesign', '--force', '--sign', signing_cert, '--verbose', filepath])


# Configurations
SIGNING_CERT = "Apple Development: Jason Bond (XF7K2AK9HM)"
APP_NAME = "Karang Lestari"
RUST_CRATE_NAME = "karang_lestari"
APP_FILE = f"build/mac/{APP_NAME}.app"
APP_RESOURCES_DIR = f"{APP_FILE}/Contents/Resources"
APP_MACOS_DIR = f"{APP_FILE}/Contents/MacOS"
SOURCE_IMAGE = "assets/icons/icon_1024x1024.png"

# Build the Rust application
print("Building Rust application...")
run_command(["cargo", "build", "--release", "--target", "aarch64-apple-darwin"])

# Bundle the application
os.makedirs(APP_MACOS_DIR, exist_ok=True)
os.makedirs(APP_RESOURCES_DIR, exist_ok=True)

shutil.copy(f"target/aarch64-apple-darwin/release/{RUST_CRATE_NAME}",
            f"{APP_MACOS_DIR}/{RUST_CRATE_NAME}")

# Copy assets
shutil.copytree("assets", f"{APP_MACOS_DIR}/assets", dirs_exist_ok=True)

# Generate app icons
iconset_path = "AppIcon.iconset"
os.makedirs(iconset_path, exist_ok=True)
sizes = [16, 32, 128, 256, 512]
for size in sizes:
    scale1 = f"{size}x{size}"
    scale2 = f"{size * 2}x{size * 2}"
    run_command(["sips", "-z", str(size), str(size), SOURCE_IMAGE, "--out", f"{iconset_path}/icon_{scale1}.png"])
    if size != 512:  # 512x512@2x is just the source image
        run_command(
            ["sips", "-z", str(size * 2), str(size * 2), SOURCE_IMAGE, "--out", f"{iconset_path}/icon_{scale2}.png"])
shutil.copy(SOURCE_IMAGE, f"{iconset_path}/icon_512x512@2x.png")
run_command(["iconutil", "-c", "icns", iconset_path])
shutil.copy("AppIcon.icns", APP_RESOURCES_DIR)

# Copy Info.plist
shutil.copy("meta/Info.plist", f"{APP_FILE}/Contents/Info.plist")

# Sign the application
run_command(["codesign", "--entitlements", "meta/entitlements.plist", "--deep", "--force", "--verify", "--verbose",
             "--options", "runtime", "--sign", SIGNING_CERT, APP_FILE])

print("Build and packaging complete.")

# Prepare installer
dmg_path = "build/karang_lestari_mac.dmg"
if os.path.exists(dmg_path):
    os.remove(dmg_path)
print("Preparing installer...")
run_command(["create-dmg",
             "--volname", "Karang Lestari Installer",
             "--volicon", "AppIcon.icns",
             "--background", "meta/installer_bg.png",
             "--window-size", "800", "400",
             "--icon-size", "128",
             "--icon", f"{APP_NAME}.app", "200", "200",
             "--hide-extension", f"{APP_NAME}.app",
             "--app-drop-link", "600", "200",
             dmg_path, "build/mac/"])

# Cleanup
shutil.rmtree(iconset_path)
os.remove("AppIcon.icns")
