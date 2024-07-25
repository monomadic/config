#!python3

import argparse
import os
import time


def write_test(path, size):
    data = os.urandom(size)  # Generate random data
    start_time = time.time()
    with open(path, "wb") as f:
        f.write(data)
    end_time = time.time()
    return end_time - start_time


def read_test(path):
    start_time = time.time()
    with open(path, "rb") as f:
        f.read()
    end_time = time.time()
    return end_time - start_time


def calculate_speed(size, elapsed_time):
    speed = size / (1024 * 1024) / elapsed_time  # MB/s
    return speed


def print_colored(text, color_code):
    print(f"\033[{color_code}m{text}\033[0m")


def main():
    parser = argparse.ArgumentParser(description="Measure read/write speed of a drive.")
    parser.add_argument(
        "--path",
        type=str,
        default=os.getcwd(),
        help="Path of the drive to test. Default is the current working directory.",
    )
    parser.add_argument(
        "--size",
        type=int,
        default=100,
        help="Size of the test file in MB. Default is 100MB.",
    )
    args = parser.parse_args()

    test_file_path = os.path.join(args.path, "test_file.bin")
    size_in_bytes = args.size * 1024 * 1024

    print_colored("💾 Drive Speed Test", "36")
    print_colored("──────────────────────────", "36")

    print_colored(f"📂 Writing {args.size}MB test file...", "34")
    write_time = write_test(test_file_path, size_in_bytes)
    write_speed = calculate_speed(size_in_bytes, write_time)
    print_colored(f"⚡ Write Speed: {write_speed:.2f} MB/s", "32")

    print_colored("📂 Reading test file...", "34")
    read_time = read_test(test_file_path)
    read_speed = calculate_speed(size_in_bytes, read_time)
    print_colored(f"⚡ Read Speed: {read_speed:.2f} MB/s", "32")

    os.remove(test_file_path)

    print_colored("──────────────────────────", "36")
    print_colored("✅ Test Completed Successfully!", "36")


if __name__ == "__main__":
    main()
