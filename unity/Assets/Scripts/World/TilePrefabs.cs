using UnityEngine;

/// A script to store a mapping of the prefab we use for each biome. Unity
/// is shitty at editing dictionaries or lists in the inspector so this is the
/// easiest way that we can assign one prefab per biome from the inspector.
/// This script should be instantiated once, on the world game object.
public class TilePrefabs : MonoBehaviour
{
    [SerializeField]
    private GameObject prefabOcean;
    [SerializeField]
    private GameObject prefabCoast;
    [SerializeField]
    private GameObject prefabSnow;
    [SerializeField]
    private GameObject prefabDesert;
    [SerializeField]
    private GameObject prefabAlpine;
    [SerializeField]
    private GameObject prefabJungle;
    [SerializeField]
    private GameObject prefabForest;
    [SerializeField]
    private GameObject prefabPlains;


    public GameObject GetBiomePrefab(string biome)
    {
        switch (biome)
        {
            case "ocean":
                return this.prefabOcean;
            case "coast":
                return this.prefabCoast;
            case "snow":
                return this.prefabSnow;
            case "desert":
                return this.prefabDesert;
            case "alpine":
                return this.prefabAlpine;
            case "jungle":
                return this.prefabJungle;
            case "forest":
                return this.prefabForest;
            case "plains":
                return this.prefabPlains;
            default:
                Debug.LogWarningFormat("Unknown biome: {0}", biome);
                return null;
        }
    }
}
