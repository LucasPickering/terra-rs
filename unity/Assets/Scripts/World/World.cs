using System;
using System.Collections.Generic;
using UnityEngine;

[Serializable]
public struct World
{
    public List<Tile> tiles;
}

[Serializable]
public struct Tile
{
    public const float MIN_ELEVATION = -100f;// TODO remove this one elevations are zero-based
    /// The distance from the center of a tile to any vertex, in world coordinates
    public const float PIXEL_RADIUS = 1.0f;
    public const float SCALE_Y = 0.1f;

    // Serialized fields, which are loaded from the generated world
    public HexPoint position;
    public float elevation;
    public string biome;

    /// The height of this tile in a zero-based scale
    public float height => (this.elevation - MIN_ELEVATION) * SCALE_Y;

    /// Convert a tile position from hex coordinates to Unity world coordinates.
    /// This returns the position of the **bottom center** of the tile.
    public Vector3 GetBottomWorldPosition()
    {
        // fuckin math, yo
        var baseX = (float)this.position.x * 1.5f;
        var baseZ = ((float)this.position.x / 2.0f + (float)this.position.y) * ((float)-Math.Sqrt(3.0));
        return new Vector3(baseX * PIXEL_RADIUS, 0, baseZ * PIXEL_RADIUS);
    }

    public override string ToString()
    {
        return String.Format("Tile {0}", this.position);
    }
}

[Serializable]
public struct HexPoint
{
    public int x;
    public int y;
    public int z
    {
        get
        {
            // https://www.redblobgames.com/grids/hexagons/#coordinates-cube
            return -this.x - this.y;
        }
    }

    public HexPoint(int x, int y)
    {
        this.x = x;
        this.y = y;
    }

    /// Get the distance between this position and another one, in terms of
    /// whole steps. This would be the number of tiles you'd have to cross to
    /// get from A to B, including the destination (so an adjacent tile is
    /// a distance of 1).
    public int DistanceTo(HexPoint other)
    {
        // https://www.redblobgames.com/grids/hexagons/#distances
        return (Math.Abs(this.x - other.x) + Math.Abs(this.y - other.y) + Math.Abs(this.z - other.z)) / 2;
    }

    public override bool Equals(object obj)
    {
        return obj is HexPoint point && this == point;
    }

    public override int GetHashCode()
    {
        // Since z is derived from x and y, we don't have to hash it
        int hashCode = 373119288;
        hashCode = hashCode * -1521134295 + x.GetHashCode();
        hashCode = hashCode * -1521134295 + y.GetHashCode();
        return hashCode;
    }

    public override string ToString() => String.Format("({0}, {1}, {2})", this.x, this.y, this.z);

    // Since z is derived from x and y, we don't have to compare it
    public static bool operator ==(HexPoint a, HexPoint b) => a.x == b.x && a.y == b.y;
    public static bool operator !=(HexPoint a, HexPoint b) => !(a == b);
    public static HexPoint operator +(HexPoint a, HexPoint b) => new HexPoint(a.x + b.x, a.y + b.y);
}

[Serializable]
public enum HexDirection
{
    Up,
    UpRight,
    DownRight,
    Down,
    DownLeft,
    UpLeft,
}

public static class HexDirectionExtensions
{
    /// Get the direction opposite this one
    public static HexDirection Opposite(this HexDirection direction)
    {
        switch (direction)
        {
            case HexDirection.Up:
                return HexDirection.Down;
            case HexDirection.UpRight:
                return HexDirection.DownLeft;
            case HexDirection.DownRight:
                return HexDirection.UpLeft;
            case HexDirection.Down:
                return HexDirection.Up;
            case HexDirection.DownLeft:
                return HexDirection.UpRight;
            case HexDirection.UpLeft:
                return HexDirection.DownRight;
            default:
                throw new NotSupportedException(String.Format("Unknown hex direction: {0}", direction));
        }
    }

    /// Convert this direction into a 1-unit offset. If you add the offset
    /// to another hex point, it will move that hex point one unit in this
    /// direction.
    public static HexPoint ToOffset(this HexDirection direction)
    {
        switch (direction)
        {
            case HexDirection.Up:
                return new HexPoint(0, 1);
            case HexDirection.UpRight:
                return new HexPoint(1, 0);
            case HexDirection.DownRight:
                return new HexPoint(1, -1);
            case HexDirection.Down:
                return new HexPoint(0, -1);
            case HexDirection.DownLeft:
                return new HexPoint(-1, 0);
            case HexDirection.UpLeft:
                return new HexPoint(-1, 1);
            default:
                throw new NotSupportedException(String.Format("Unknown hex direction: {0}", direction));
        }
    }
}
