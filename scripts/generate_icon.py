"""Generate minimal monochrome app icons for Tauri."""
from pathlib import Path
from PIL import Image, ImageDraw

root = Path(__file__).resolve().parents[1] / "src-tauri" / "icons"
root.mkdir(parents=True, exist_ok=True)
image = Image.new("RGBA", (1024, 1024), (255, 255, 255, 0))
draw = ImageDraw.Draw(image)
draw.rounded_rectangle((64, 64, 960, 960), radius=190, fill=(24, 24, 24, 255))
draw.rounded_rectangle((245, 268, 779, 375), radius=19, fill=(255, 255, 255, 255))
draw.rounded_rectangle((459, 268, 565, 764), radius=19, fill=(255, 255, 255, 255))
image.save(root / "icon.png")
image.resize((32, 32), Image.Resampling.LANCZOS).save(root / "32x32.png")
image.resize((128, 128), Image.Resampling.LANCZOS).save(root / "128x128.png")
image.save(root / "icon.ico", sizes=[(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)])
