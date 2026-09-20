#!python3

import argparse
import os
from struct import unpack


def read_string(fp, offset):
    fp.seek(offset)
    fname = b""
    while 1:
        b = fp.read(1)
        if b == b"\x00":
            break
        fname += b
    return fname.decode()


def main(args):
    print(f"Reading {args.input}!")
    nspFile = open(os.path.abspath(args.input), "rb")
    nspFile.seek(0)

    if nspFile.read(4) != b"PFS0":
        print("Invalid file magic.")
        os._exit(4)

    numFiles = unpack("<I", nspFile.read(4))[0]  # number of files in archive
    print(f"numFiles: {numFiles}")

    stableSize = unpack("<I", nspFile.read(4))[0]  # size of string table
    print(f"stableSize: {stableSize}")

    nspFile.read(4)  # skip seperator

    # file table entries
    fspecs = []
    for _ in range(numFiles):
        data_offset = unpack("<Q", nspFile.read(8))[0]
        data_size = unpack("<Q", nspFile.read(8))[0]
        name_offset = unpack("<I", nspFile.read(4))[0]
        nspFile.read(4)  # skip seperator
        fspecs.append((data_offset, data_size, name_offset))

    print(fspecs)

    # store info about name, size for pretty printing
    fnames, fsizes = [], []
    for fspec in fspecs:
        fnames.append(read_string(nspFile, fspec[2]))
        fsizes.append(str(fspec[1]))

    print(fnames)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="NSP Extractor")
    parser.add_argument("-i", "--input", type=str, help="NSP file")
    args = parser.parse_args()
    main(args)
