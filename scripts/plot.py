#!/usr/bin/env python3

import argparse
import cbor
import matplotlib.pyplot as plt


def main():
    parser = argparse.ArgumentParser("Plot characteristics of a Terra world")
    parser.add_argument("world", help="The world to plot, as a CBOR file")
    args = parser.parse_args()

    with open(args.world, "rb") as f:
        world = cbor.load(f)
    plot_elevations(world)


def plot_elevations(world):
    elevations = [tile["elevation"] for tile in world["tiles"]]
    plt.hist(elevations, bins=20)
    plt.show()


if __name__ == "__main__":
    main()
