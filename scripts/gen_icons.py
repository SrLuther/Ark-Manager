from PIL import Image
import os
import struct

src = r"C:\Users\Ciano\Documents\Ark Manager\src\assets\logo\ark_manager_logo.png"
icons_dir = r"C:\Users\Ciano\Documents\Ark Manager\src-tauri\icons"
logo_dir = r"C:\Users\Ciano\Documents\Ark Manager\src\assets\logo"

os.makedirs(icons_dir, exist_ok=True)

img = Image.open(src).convert("RGBA")
print(f"Original: {img.size} {img.mode}")

def resize(image, size):
    return image.resize((size, size), Image.LANCZOS)

# --- Tauri required icons ---
resize(img, 32).save(os.path.join(icons_dir, "32x32.png"))
resize(img, 128).save(os.path.join(icons_dir, "128x128.png"))
resize(img, 256).save(os.path.join(icons_dir, "128x128@2x.png"))

# --- ICO (16, 32, 48, 256) ---
ico_sizes = [16, 32, 48, 256]
ico_images = [resize(img, s) for s in ico_sizes]
ico_images[0].save(
    os.path.join(icons_dir, "icon.ico"),
    format="ICO",
    sizes=[(s, s) for s in ico_sizes],
    append_images=ico_images[1:]
)

# --- Frontend logo variants ---
resize(img, 64).save(os.path.join(logo_dir, "logo-64.png"))
resize(img, 128).save(os.path.join(logo_dir, "logo-128.png"))
resize(img, 256).save(os.path.join(logo_dir, "logo-256.png"))
# Keep original as logo-full.png
img.save(os.path.join(logo_dir, "logo-full.png"))

print("Variações geradas com sucesso!")
print(f"  icons/32x32.png")
print(f"  icons/128x128.png")
print(f"  icons/128x128@2x.png (256x256)")
print(f"  icons/icon.ico (16, 32, 48, 256)")
print(f"  logo/logo-64.png")
print(f"  logo/logo-128.png")
print(f"  logo/logo-256.png")
print(f"  logo/logo-full.png")

img.close()
