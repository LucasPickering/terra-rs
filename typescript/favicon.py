#!/usr/bin/env python3

import argparse
import subprocess
from glob import glob

FILE_NAME_FMT = "favicon-{}.png"


def run_cmd(cmd):
    print(" ".join(cmd))
    subprocess.check_output(cmd)


def main():
    parser = argparse.ArgumentParser(description="Make a favicon")
    parser.add_argument("input_file")
    parser.add_argument("--output-file", "-o", default="favicon.ico")
    parser.add_argument("--max-size", "-s", type=int, default=256)
    parser.add_argument("--min-size", type=int, default=16)
    args = parser.parse_args()

    # Create the initial file
    size = args.max_size
    max_size_file = FILE_NAME_FMT.format(size)
    all_files = [max_size_file]
    try:
        run_cmd(
            [
                "inkscape",
                "-z",
                "-w",
                str(size),
                "-h",
                str(size),
                args.input_file,
                "-e",
                max_size_file,
            ]
        )
        while size > args.min_size:
            size //= 2
            file = FILE_NAME_FMT.format(size)
            run_cmd(
                [
                    "convert",
                    max_size_file,
                    "-background",
                    "none",
                    "-resize",
                    f"{size}x{size}",
                    file,
                ]
            )
            all_files.append(file)
        run_cmd(["convert"] + all_files + [args.output_file])
    finally:
        run_cmd(["rm"] + glob(FILE_NAME_FMT.format("*")))


if __name__ == "__main__":
    main()
