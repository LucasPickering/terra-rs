using System;
using System.Linq;
using System.Collections.Generic;
using UnityEngine;
using UnityEditor;

/// A custom editor to allow generating (and deleting) a world of tiles from
/// within the editor
[CustomEditor(typeof(WorldController))]
public class WorldEditor : Editor
{
    public override void OnInspectorGUI()
    {
        base.OnInspectorGUI();
        var worldController = (WorldController)this.target;

        if (GUILayout.Button("Clear World"))
        {
            worldController.ClearWorld();
        }
        if (GUILayout.Button("Generate World"))
        {
            worldController.GenerateWorld();
        }
    }
}
