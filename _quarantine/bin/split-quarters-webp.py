#!python3

import sys

from PIL import Image

image_paths = sys.argv[1:]

for image_path in image_paths:
    img = Image.open(image_path)
    width, height = img.size

    quarter_width = width // 2
    quarter_height = height // 2

    name, _ = image_path.split(".")

    img1 = img.crop((0, 0, quarter_width, quarter_height))
    img1.save(f"{name}-01.webp")

    img2 = img.crop((quarter_width, 0, width, quarter_height))
    img2.save(f"{name}-02.webp")

    img3 = img.crop((0, quarter_height, quarter_width, height))
    img3.save(f"{name}-03.webp")

    img4 = img.crop((quarter_width, quarter_height, width, height))
    img4.save(f"{name}-04.webp")
