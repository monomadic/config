#!python3

import argparse
import os
import subprocess


def expand_path(path):
    return os.path.expanduser(os.path.expandvars(path))


def update_or_clone_repositories(input_file):
    if not os.path.exists(input_file):
        print(f"The file '{input_file}' does not exist.")
        return

    with open(input_file, "r") as f:
        lines = f.readlines()

    for line in lines:
        folder_path, git_url = line.strip().split(",")
        folder_path = expand_path(folder_path.strip())
        git_url = git_url.strip()

        if os.path.exists(folder_path):
            print(f"Updating repository in {folder_path}...")
            subprocess.run(["git", "pull"], cwd=folder_path)
        else:
            print(f"Cloning repository from {git_url} to {folder_path}...")
            subprocess.run(["git", "clone", "--depth=1", git_url, folder_path])


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="Syncs git repositories to a local folder based on a text file."
    )
    parser.add_argument(
        "file",
        type=str,
        help="text file containing remote repositories and local directories to sync to",
        default="$HOME/config/repositories.txt",
    )
    args = parser.parse_args()
    input_file = args.file
    input_file = expand_path(input_file.strip())
    update_or_clone_repositories(input_file)
