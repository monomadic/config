#!/usr/bin/env python3

from PIL import Image

def split_image(image_path):
    image = Image.open(image_path)
    width, height = image.size

    # Ensure the image dimensions are divisible by 2
    assert width % 2 == 0, "Image width must be divisible by 2"
    assert height % 2 == 0, "Image height must be divisible by 2"

    half_width = width // 2
    half_height = height // 2

    top_left = image.crop((0, 0, half_width, half_height))
    top_right = image.crop((half_width, 0, width, half_height))
    bottom_left = image.crop((0, half_height, half_width, height))
    bottom_right = image.crop((half_width, half_height, width, height))

    # Save each quarter
    top_left.save('top_left.png')
    top_right.save('top_right.png')
    bottom_left.save('bottom_left.png')
    bottom_right.save('bottom_right.png')

# Call the function with the image path
split_image('your_image.png')
