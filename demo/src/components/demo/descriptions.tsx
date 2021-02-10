import React from "react";
import Link from "../Link";

/**
 * The help text we show for each element on the config page. This object
 * layout generally aligns with WorldConfigObject, but there are some
 * differences. It should exactly align with the layout of the config page
 * though.
 */
const descriptions = {
  general: {
    root: "",
    seed: (
      <>
        Seed used for all random number generation. See{" "}
        <Link to="https://en.wikipedia.org/wiki/Pseudorandom_number_generator">
          pseudorandom number generation
        </Link>
        . You can enter any text here, and it will be converted to a seed value.
      </>
    ),
    radius: (
      <>
        The size of the world, as a distance from the center to the edge. A
        radius of 0 means the world is just 1 tile.
      </>
    ),
  },
  edge_buffer: {
    root: (
      <>
        The edge buffer is an area around the edge of the map where elevations
        are suppresed in order to guarantee that the edge is entirely ocean, so
        that no land mass gets cut off. You can imagine the edge buffer as a
        ring of tiles around the edge of the map that slopes down from its inner
        edge to its outer edge.
      </>
    ),
    edge_buffer_fraction: (
      <>
        The fraction of the map that constitutes the edge buffer. 0.1 means the
        outer 10%, 0.2 is the outer 20%, etc. Set to 0 to disable the edge
        buffer.
      </>
    ),
    edge_buffer_exponent: (
      <>
        <p>
          The exponent applied to the edge buffer scaling function.{" "}
          <code>e=1.0</code> means the edge buffer will generally be a linear
          descent from the inner edge of the buffer to the outer edge.
        </p>
        <p>
          <code>e&lt;1.0</code> means the gradient will be more gradual towards
          the inside and more aggressive at the end.
        </p>
        <p>
          <code>e&gt;1.0</code> mean the gradient will be steepest at the inner
          edge and will level out towards the outer edge.
        </p>
      </>
    ),
  },
  elevation: {
    root: (
      <>
        <p>
          Tile elevations are generated via a{" "}
          <Link to="https://en.wikipedia.org/wiki/Perlin_noise">
            Perlin noise function
          </Link>
          . The function is a composite of multiple "octaves", where each octave
          is a cyclic function with a set frequency. The resulting function is
          the sum of each octave.
        </p>
        <p>
          These settings control the behavior of the base function (the lowest
          octave), and how subsequent octaves are derived from there. We also
          apply post-process each elevation by applying an exponent to the
          output values of the noise function. In math terms, the elevation
          function looks something like:
        </p>

        <p>
          <code>
            Elev(x, y) = P<sub>o,f,l,p</sub>(x, y)<sup>exp</sup>
          </code>
        </p>
      </>
    ),
    octaves: (
      <>
        The number of octaves in the noise function. Fewer octaves tends to give
        more varied results but appears very artificial. Add more octaves to
        make more realistic-looking terrain.
      </>
    ),
    frequency: (
      <>
        The frequency of the lowest octave. Lower frequencies will give smoother
        terrain, where peaks are further apart. Higher frequencies give more
        jagged terrain.
      </>
    ),
    lacunarity: (
      <>
        The difference in frequency between one octave and the next. E.g. with{" "}
        <code>l=2.0</code> if the lowest octave has <code>f=1.0</code>, then the
        next two octaves will be <code>f=3.0</code> and <code>f=5.0</code>.
      </>
    ),
    persistence: (
      <>
        The ratio of amplitude between one octave and the next. With{" "}
        <code>p=1.0</code>, all octaves will have the same amplitude. With{" "}
        <code>p&lt;1.0</code>, the amplitude will diminish from one octave to
        the next, which is typically what you want. With <code>p&gt;1.0</code>,
        the amplitude will increase from one octave to the next, which is rather
        weird but I won't stop you.
      </>
    ),
    exponent: (
      <>
        The exponent of the polynomial post-processing function on each
        elevation. With <code>exp=1.0</code>, each elevation will simply be the
        output of the noise function. With <code>e&gt;1.0</code>, elevations
        will generally be pushed down with the steepest suppression on the
        highest tiles. With <code>e&lt;1.0</code>, elevations will generally be
        elevated, again with the biggest difference occuring on the highest
        tiles.
      </>
    ),
  },
  rainfall: {
    root: (
      <>
        Terra simulates rainfall on each tile in order to determine how humid
        each tile is (which affects its biome) and how much runoff originates on
        that tile, which is used to form rivers and lakes. The algorithm for
        rainfall simulation is roughly:
        <ol>
          <li>
            Pick a prevailing wind direction, which is uniform over the entire
            world.
          </li>
          <li>
            Initialize a "cloud line", which starts at the most upwind point
            tiles in the world and spans the width of the entire world. The
            cloud line will be perpendicular to the prevailing wind direction.
          </li>
          <li>
            For each tile in the cloud line, accumulate some amount of
            evaporation from nearby tiles that are directly under the clouds.
          </li>
          <li>
            For each tile in the cloud line, drop some fraction of that cloud
            tile's accumulated evaporation onto the tile directly below.
          </li>
          <li>Push the cloud line downwind one tile and repeat.</li>
        </ol>
      </>
    ),
    evaporation_default: (
      <>
        The amount of evaporation (in m³) that an ocean tile provides to the
        cloud directly above it.
      </>
    ),
    evaporation_land_scale: (
      <>
        The amount of evaporation that a land tile provides to the cloud
        directly above it, as a fraction of the default evaporation value. E.g.
        if the default value is 10 m³ and the land scale is 0.1, then each land
        tile will provide 1 m³ to their nearest friendly neighborhood cloud.
      </>
    ),
    evaporation_spread_distance: (
      <>
        The distance (in tiles) that evaporation spreads, perpendicular to the
        wind. E.g. if we consider the wind direction to be *forward*, then this
        is the distance to the left and right that a particular tile's
        evaporation will spread. This is a smoothing mechanism that makes
        precipitation patterns appear smoother/more natural.
      </>
    ),
    evaporation_spread_exponent: (
      <>
        Exponent to apply while calculating spread diminishment. If the exponent
        is 1.0, then evaporation spread will be linear, meaning the amount of
        evaporation that one tile will receive from another tile that is{" "}
        <code>n</code> steps away will be proportional to <code>n</code>. If
        this is <code>&lt;1</code>, then spreading will be biased towards the
        center, and if <code>&gt;1</code> will be biased towards the outer
        edges.
      </>
    ),
    rainfall_fraction_limit: (
      <>
        The maximum fraction of a cloud's rainfall that can be dropped on any
        particular tile. E.g. if this is 0.01, then a cloud can drop at most 1%
        of its held water on a single tile. This value should typically be
        pretty small, to allow water to spread over huge tracts of land.
      </>
    ),
  },
  geo_feature: {
    root: (
      <>
        Geographic features are features generated on tiles as a result of prior
        generated attributes. For example, we generate lakes based on
        accumulated runoff, and rivers based on total traversed runoff.
      </>
    ),
    lake_runoff_threshold: (
      <>
        The amount of runoff needed to accumulate on a tile (in m³) for it to
        become a lake.
      </>
    ),
    river_runoff_traversed_threshold: (
      <>
        The amount of runoff needed to <em>pass over</em> a tile for it to
        become a river. Unlike lakes, where we just look at accumumlated runoff,
        for rivers we look at all the runoff that flows over a tile. As such,
        this threshold should be a lot higher than the lake threshold.
      </>
    ),
  },
};

export default descriptions;
