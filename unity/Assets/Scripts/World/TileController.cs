using System;
using System.Collections;
using System.Collections.Generic;
using UnityEngine;

/// Attached to a single tile object, stores game data for it
public class TileController : MonoBehaviour
{
    public GameObject hexagonModel;
    public Tile tile;

    public Dictionary<HexDirection, TileController> neighbors
    {
        get => this._neighbors;
    }

    // This is populated by the world generator during startup
    private Dictionary<HexDirection, TileController> _neighbors = new Dictionary<HexDirection, TileController>();

    /// TODO docs
    /// TODO rename
    public void InitStuff(float verticalScale)
    {
        var height = this.tile.height * verticalScale;

        // Set position. We treat the top of the hexagon as the tile's origin,
        // which makes it easy to place stuff on top
        this.gameObject.transform.position = tile.GetBottomWorldPosition() + new Vector3(0f, height, 0f);

        // Y scale based on elevation. We only want to scale the hexagon itself,
        // not any of the extra stuff on top. We use a negative scale so the
        // tile goes down, to maintain the "top as origin" behavior
        this.hexagonModel.transform.localScale = new Vector3(1.0f, -height, 1.0f);
    }

    /// Register the given tile as a neighbor to this tile. This will store
    /// the neighbor in this tile's map of neighbor's, AND do the reciprocal
    /// operation on the neighbor: register this tile in ITS neighbor map. If
    /// the tiles have already been linked as neighbors, this does nothing.
    public bool AddNeighbor(TileController neighbor, HexDirection direction)
    {
        if (!this.neighbors.ContainsKey(direction))
        {
            this.neighbors.Add(direction, neighbor);
            neighbor.neighbors.Add(direction.Opposite(), this);
            return true;
        }
        return false;
    }

    public override string ToString()
    {
        return String.Format("TileController {0}", this.tile.position);
    }
}
