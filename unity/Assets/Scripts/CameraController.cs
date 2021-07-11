using UnityEngine;

public class CameraController : MonoBehaviour
{
    public float moveSpeed = 120.0f;
    public float rotateSpeed = 150.0f;
    public float zoomSpeed = 30.0f;
    [Range(0.1f, 5f)]
    public float minCameraSize = 1f;
    [Range(10f, 50f)]
    public float maxCameraSize = 30f;
    public KeyCode keyForward = KeyCode.W;
    public KeyCode keyBackward = KeyCode.S;
    public KeyCode keyLeft = KeyCode.A;
    public KeyCode keyRight = KeyCode.D;
    public KeyCode keyRotateLeft = KeyCode.Q;
    public KeyCode keyRotateRight = KeyCode.E;


    // Update is called once per frame
    void Update()
    {

        // Adjust camera zoom based on scroll wheel
        var camera = gameObject.GetComponent<Camera>();
        var cameraSize = camera.orthographicSize;
        cameraSize += -Input.mouseScrollDelta.y * zoomSpeed * Time.deltaTime;
        camera.orthographicSize = Mathf.Clamp(cameraSize, minCameraSize, maxCameraSize);

        // Rotate the camera based on rotate keys
        var yRotation = GetYRotation();
        if (yRotation != 0f)
        {
            // This code makes us rotate around the center of the screen
            // instead of origin. Feels weird right now but we may want it later
            // var plane = new Plane(Vector3.up, Vector3.zero);
            // var ray = Camera.main.ViewportPointToRay(new Vector3(0.5f, 0.5f));
            // float enter = 0.0f;
            // plane.Raycast(ray, out enter);
            // var rotateOrigin = ray.GetPoint(enter);

            transform.RotateAround(Vector3.zero, Vector3.up, yRotation);
        }

        // Translate the camera based on pan keys. We need to rotate the
        // translation so that forward is the direction the camera is pointing,
        // not just positive z (same goes for the other 3 directions)
        var velocity = Quaternion.Euler(0, transform.localRotation.eulerAngles.y, 0) * GetTranslation();
        if (velocity.sqrMagnitude > 0)
        {
            transform.position += velocity;
        }
    }

    private Vector3 GetTranslation()
    {
        Vector3 velocity = new Vector3();
        if (Input.GetKey(keyForward))
        {
            velocity += new Vector3(0, 0, 1);
        }
        if (Input.GetKey(keyBackward))
        {
            velocity += new Vector3(0, 0, -1);
        }
        if (Input.GetKey(keyLeft))
        {
            velocity += new Vector3(-1, 0, 0);
        }
        if (Input.GetKey(keyRight))
        {
            velocity += new Vector3(1, 0, 0);
        }
        return velocity * moveSpeed * Time.deltaTime;
    }

    private float GetYRotation()
    {
        float yRotation = 0f;
        if (Input.GetKey(keyRotateLeft))
        {
            yRotation += 1f;
        }
        if (Input.GetKey(keyRotateRight))
        {
            yRotation -= 1f;
        }
        return yRotation * rotateSpeed * Time.deltaTime;
    }
}
