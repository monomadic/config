#!/usr/bin/env python3

import cv2
import numpy as np
import matplotlib.pyplot as plt

def euclidean_distance(color1, color2):
    """Calculate the Euclidean distance between two colors."""
    return np.sqrt(np.sum((color1 - color2) ** 2))

def process_image(img, min_block_size, max_block_size, color_threshold):
    """Process an image by coloring blocks of similar colors with their average color."""
    # Prepare the output image
    output_img = img.copy()

    # Get the dimensions of the image
    height, width, _ = img.shape

    # Iterate over the image with a step size equal to the minimum block size
    for y in range(0, height, min_block_size):
        for x in range(0, width, min_block_size):

            # Try different block sizes
            for block_size in range(min_block_size, max_block_size + 1):
                # Ensure the block fits within the image
                if y + block_size <= height and x + block_size <= width:
                    # Extract the block
                    block = img[y:y+block_size, x:x+block_size]

                    # Calculate the average color of the block
                    avg_color = np.mean(block, axis=(0, 1))

                    # Check if each pixel in the block has a color close to the average color
                    color_diffs = np.apply_along_axis(euclidean_distance, 2, block, avg_color)

                    # If all pixels in the block pass the color check, color the block
                    if np.all(color_diffs < color_threshold):
                        output_img[y:y+block_size, x:x+block_size] = avg_color

    return output_img

def main():
    parser = argparse.ArgumentParser(description='Quantize pixels')
    parser.add_argument('input', type=str, help='image')

    args = parser.parse_args()

    # Load the image
    img = cv2.imread(args.input)

    # Set parameters
    min_block_size = 6
    max_block_size = 7
    color_threshold = 50  # This might need to be adjusted

    # Process the image
    output_img = process_image(img, min_block_size, max_block_size, color_threshold)

    # Convert the output image to RGB for displaying
    output_img_rgb = cv2.cvtColor(output_img, cv2.COLOR_BGR2RGB)

    # Display the output image
    plt.imshow(output_img_rgb)
    plt.axis('off')
    plt.show()

if __name__ == "__main__":
    main()
