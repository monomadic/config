#!/usr/bin/env python3
# pip3 install --break-system-packages click rich
import os
import shutil
import time

import click
from rich.console import Console
from rich.panel import Panel
from rich.progress import (
    BarColumn,
    Progress,
    SpinnerColumn,
    TaskProgressColumn,
    TextColumn,
)
from rich.table import Table

console = Console()


def write_test(path, size, progress, task):
    chunk_size = 1024 * 1024  # 1 MB chunks
    data = os.urandom(chunk_size)
    written = 0
    start_time = time.time()
    with open(path, "wb") as f:
        while written < size:
            if size - written < chunk_size:
                chunk_size = size - written
                data = os.urandom(chunk_size)
            f.write(data)
            written += chunk_size
            progress.update(task, completed=written)
    end_time = time.time()
    return end_time - start_time


def read_test(path, size, progress, task):
    chunk_size = 1024 * 1024  # 1 MB chunks
    read = 0
    start_time = time.time()
    with open(path, "rb") as f:
        while read < size:
            if size - read < chunk_size:
                chunk_size = size - read
            f.read(chunk_size)
            read += chunk_size
            progress.update(task, completed=read)
    end_time = time.time()
    return end_time - start_time


def calculate_speed(size, elapsed_time):
    return size / (1024 * 1024) / elapsed_time  # MB/s


def check_conditions(path, size):
    try:
        # Check if the directory exists
        if not os.path.isdir(path):
            raise click.ClickException(
                f"The specified path '{path}' is not a valid directory."
            )

        # Check if we have write permissions
        test_file = os.path.join(path, "test_permissions.tmp")
        try:
            with open(test_file, "w") as f:
                f.write("test")
            os.remove(test_file)
        except PermissionError:
            raise click.ClickException(
                f"You don't have write permissions in the directory '{path}'."
            )

        # Check if there's enough space
        free_space = shutil.disk_usage(path).free
        if free_space < size:
            raise click.ClickException(
                f"Not enough free space. Required: {size/(1024*1024):.2f} MB, Available: {free_space/(1024*1024):.2f} MB"
            )
    except Exception as e:
        console.print(f"[bold red]Error:[/bold red] {str(e)}")
        raise click.Abort()


@click.command()
@click.option(
    "--path",
    type=click.Path(exists=True),
    default=os.getcwd(),
    help="Path of the drive to test. Default is the current working directory.",
)
@click.option(
    "--size",
    type=int,
    default=100,
    help="Size of the test file in MB. Default is 100MB.",
)
def main(path, size):
    """Measure read/write speed of a drive."""
    size_in_bytes = size * 1024 * 1024
    test_file_path = os.path.join(path, "test_file.bin")

    console.print(Panel.fit("Drive Speed Test", style="bold cyan"))

    with console.status("[bold blue]Checking conditions...", spinner="dots") as status:
        check_conditions(path, size_in_bytes)
        status.update("[bold green]All conditions met. Starting test...")
        time.sleep(1)

    with Progress(
        SpinnerColumn(),
        TextColumn("[progress.description]{task.description}"),
        BarColumn(),
        TaskProgressColumn(),
        console=console,
    ) as progress:
        write_task = progress.add_task(
            "[cyan]Writing test file...", total=size_in_bytes
        )
        write_time = write_test(test_file_path, size_in_bytes, progress, write_task)

        read_task = progress.add_task("[cyan]Reading test file...", total=size_in_bytes)
        read_time = read_test(test_file_path, size_in_bytes, progress, read_task)

    os.remove(test_file_path)

    write_speed = calculate_speed(size_in_bytes, write_time)
    read_speed = calculate_speed(size_in_bytes, read_time)

    table = Table(title="Test Results")
    table.add_column("Operation", style="cyan")
    table.add_column("Speed", style="magenta")
    table.add_row("Write", f"{write_speed:.2f} MB/s")
    table.add_row("Read", f"{read_speed:.2f} MB/s")

    console.print(table)
    console.print("[bold green]Drive speed test completed successfully.[/bold green]")


if __name__ == "__main__":
    main()
