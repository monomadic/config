#!/usr/bin/env python3

import argparse
import math

from PIL import Image


def compare_colors(color1, color2, threshold=10):
    # Calculate Euclidean distance between two colors
    distance = math.sqrt(sum((a - b) ** 2 for a, b in zip(color1, color2)))
    return distance < threshold


def process_image(img_path):
    img = Image.open(img_path)
    pixels = img.load()

    width, height = img.size
    for i in range(width):
        for j in range(height):
            # If it's not the first pixel and its color is close to the previous one
            if i > 0 and compare_colors(pixels[i, j], pixels[i - 1, j]):
                # Set the color of this pixel the same as the previous one
                pixels[i, j] = pixels[i - 1, j]

    img.save("processed_image.png")


def main():
    parser = argparse.ArgumentParser(description="Quantize pixels")
    parser.add_argument("input", type=str, help="image")

    args = parser.parse_args()

    process_image(args.input)


if __name__ == "__main__":
    main()
