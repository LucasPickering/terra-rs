using System;
using System.Collections;
using System.Collections.Generic;
using System.Linq;
using UnityEngine;

public class WorldController : MonoBehaviour
{
    public TextAsset worldJsonFile;
    public GameObject tilePrefab;
    /// Scale factor between elevation values from Terra and Y axis values in
    /// Unity
    public float verticalScale = 1.0f;

    // Unity isn't great at serializing dictionaries, so we serialize a list of
    // all tiles, then at startup we'll build a dictionary from the list so
    // we can easily look up tiles by their position.
    [HideInInspector]
    [SerializeField]
    private List<TileController> tiles;
    private Dictionary<HexPoint, TileController> tileDict;

    void Start()
    {
        // Map the list of tiles to a dictionary. Also, link each tile to its
        // neighbors so we can traverse the world like a graph.
        this.tileDict = new Dictionary<HexPoint, TileController>();
        foreach (var tileController in this.tiles)
        {
            this.tileDict.Add(tileController.tile.position, tileController);

            // Link this tile to its neighbors
            foreach (HexDirection direction in Enum.GetValues(typeof(HexDirection)))
            {
                HexPoint neighborPos = tileController.tile.position + direction.ToOffset();
                TileController neighbor;
                if (this.tileDict.TryGetValue(neighborPos, out neighbor))
                {
                    tileController.AddNeighbor(neighbor, direction);
                }
            }
        }
    }


    public TileController GetTileByPosition(HexPoint pos)
    {
        return this.tileDict[pos];
    }

    /// Delete the tile objects for the existing world (if any exist). Meant
    /// to be called from an editor script, not at runtime!
    public void ClearWorld()
    {
        this.tiles = new List<TileController>();

        // We have to copy the children into a list before destroying them.
        // Otherwise we'd be modifying the children enumerable while iterating
        // over it, which leads to a bug where not all children get destroy
        var childrenToDestroy = this.transform.Cast<Transform>().ToList();
        foreach (Transform child in childrenToDestroy)
        {
            GameObject.DestroyImmediate(child.gameObject);
        }
    }

    /// Generate tiles objects based on the defined world file. If the a world
    /// is already present, it will be cleared first. Meant to be called from an
    /// editor script, not at runtime!
    public void GenerateWorld()
    {
        // Clear the existing world from the scene
        this.ClearWorld();

        // Load the new world from a JSON file
        var world = JsonUtility.FromJson<World>(this.worldJsonFile.ToString());
        var biomeMaterials = this.GetComponent<TilePrefabs>();

        // For each tile in the world, initialize a game object
        foreach (Tile tile in world.tiles)
        {
            var height = tile.height * this.verticalScale;

            // We position tiles from the top, and scale downwards. This makes
            // it a lot easier to place objects on top of the tiles (just make
            // it a child and you're good to go)
            var tileGameObject = Instantiate(biomeMaterials.GetBiomePrefab(tile.biome), Vector3.zero, Quaternion.identity);
            // Nest the tile under whatever object this script is attached to
            tileGameObject.transform.parent = this.transform;

            var tileController = tileGameObject.GetComponent<TileController>();
            tileController.tile = tile;
            tileController.InitStuff(this.verticalScale);
            // Register every tile in the list
            this.tiles.Add(tileController);

            tileGameObject.name = tile.ToString();
        }
    }
}
