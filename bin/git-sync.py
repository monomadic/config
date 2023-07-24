#!python3

import os
import subprocess

default_input_file = "$HOME/config/repositories.txt"


def expand_path(path):
    return os.path.expanduser(os.path.expandvars(path))


def update_or_clone_repositories(input_file):
    if not os.path.exists(input_file):
        print(f"The file '{input_file}' does not exist.")
        return

    with open(input_file, "r") as f:
        lines = f.readlines()

    for line in lines:
        git_url, folder_path = line.strip().split(",")
        folder_path = expand_path(folder_path.strip())

        if os.path.exists(folder_path):
            print(f"Updating repository in {folder_path}...")
            subprocess.run(["git", "pull"], cwd=folder_path)
        else:
            print(f"Cloning repository from {git_url} to {folder_path}...")
            subprocess.run(["git", "clone", git_url, folder_path])


if __name__ == "__main__":
    input_file = expand_path(default_input_file.strip())
    update_or_clone_repositories(input_file)
