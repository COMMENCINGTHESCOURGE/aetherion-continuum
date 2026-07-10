import json
import math
import numpy as np

def orthonormalize_matrix(m):
    """Decomposes a 4x4 column-major transformation matrix, removes scale/shear,
    and returns clean translation, rotation (quaternion), and unit scale."""
    M = np.array(m).reshape((4, 4), order='F') # Column-major
    
    # 1. Extract Translation (4th column, top 3 elements)
    translation = M[0:3, 3].tolist()
    
    # 2. Extract rotation columns and orthonormalize (Gram-Schmidt)
    c0 = M[0:3, 0]
    c1 = M[0:3, 1]
    c2 = M[0:3, 2]
    
    # Normalize c0
    l0 = np.linalg.norm(c0)
    u0 = c0 / l0 if l0 > 0 else np.array([1.0, 0.0, 0.0])
    
    # Orthogonalize c1 against u0
    proj_c1 = np.dot(c1, u0) * u0
    u1_raw = c1 - proj_c1
    l1 = np.linalg.norm(u1_raw)
    u1 = u1_raw / l1 if l1 > 0 else np.array([0.0, 1.0, 0.0])
    
    # Orthogonalize c2 against u0 and u1
    proj_c2 = np.dot(c2, u0) * u0 + np.dot(c2, u1) * u1
    u2_raw = c2 - proj_c2
    l2 = np.linalg.norm(u2_raw)
    u2 = u2_raw / l2 if l2 > 0 else np.cross(u0, u1)
    
    # Orthonormal rotation matrix
    R = np.column_stack((u0, u1, u2))
    
    # 3. Convert R to Quaternion (w, x, y, z) -> glTF expects (x, y, z, w)
    tr = np.trace(R)
    if tr > 0:
        S = math.sqrt(tr + 1.0) * 2
        qw = 0.25 * S
        qx = (R[2, 1] - R[1, 2]) / S
        qy = (R[0, 2] - R[2, 0]) / S
        qz = (R[1, 0] - R[0, 1]) / S
    elif (R[0, 0] > R[1, 1]) and (R[0, 0] > R[2, 2]):
        S = math.sqrt(1.0 + R[0, 0] - R[1, 1] - R[2, 2]) * 2
        qw = (R[2, 1] - R[1, 2]) / S
        qx = 0.25 * S
        qy = (R[0, 1] + R[1, 0]) / S
        qz = (R[0, 2] + R[2, 0]) / S
    elif R[1, 1] > R[2, 2]:
        S = math.sqrt(1.0 + R[1, 1] - R[0, 0] - R[2, 2]) * 2
        qw = (R[0, 2] - R[2, 0]) / S
        qx = (R[0, 1] + R[1, 0]) / S
        qy = 0.25 * S
        qz = (R[1, 2] + R[2, 1]) / S
    else:
        S = math.sqrt(1.0 + R[2, 2] - R[0, 0] - R[1, 1]) * 2
        qw = (R[1, 0] - R[0, 1]) / S
        qx = (R[0, 2] + R[2, 0]) / S
        qy = (R[1, 2] + R[2, 1]) / S
        qz = 0.25 * S
        
    rotation = [qx, qy, qz, qw] # glTF quaternion format
    scale = [1.0, 1.0, 1.0]
    
    return translation, rotation, scale

def sanitize_and_articulate():
    gltf_path = r"C:\Users\dasha\Projects\aetherion-continuum\leiei.gltf"
    out_path = r"C:\Users\dasha\Projects\aetherion-continuum\leiei_sanitized.gltf"
    
    with open(gltf_path, "r", encoding="utf-8") as f:
        data = json.load(f)
        
    nodes = data.get("nodes", [])
    
    print("--- Sanitizing Transforms ---")
    for i, node in enumerate(nodes):
        name = node.get("name", "Unnamed")
        print(f"Sanitizing Node {i}: {name}")
        
        # If node has matrix, decompose it to orthonormal translation/rotation/scale
        if "matrix" in node:
            t, r, s = orthonormalize_matrix(node["matrix"])
            node.pop("matrix")
            node["translation"] = t
            node["rotation"] = r
            node["scale"] = s
        else:
            # Force uniform scale of 1.0
            node["scale"] = [1.0, 1.0, 1.0]
            # Ensure rotation is present or unit quaternion
            if "rotation" in node:
                # Normalize rotation to prevent quaternion drift
                r = node["rotation"]
                norm = math.sqrt(sum(x*x for x in r))
                if norm > 0:
                    node["rotation"] = [x/norm for x in r]
            else:
                node["rotation"] = [0.0, 0.0, 0.0, 1.0]
            
            if "translation" not in node:
                node["translation"] = [0.0, 0.0, 0.0]
                
    # --- Articulation Pass ---
    # Add Propeller and Rudder nodes to the glTF
    print("--- Adding Articulated Appendage Nodes ---")
    
    prop_node = {
        "name": "leilei_propeller",
        "translation": [0.0, -0.4, -2.4],
        "rotation": [0.0, 0.0, 0.0, 1.0],
        "scale": [1.0, 1.0, 1.0]
    }
    
    rudder_node = {
        "name": "leilei_rudder",
        "translation": [0.0, -0.2, -2.8],
        "rotation": [0.0, 0.0, 0.0, 1.0],
        "scale": [1.0, 1.0, 1.0]
    }
    
    # Append these nodes to nodes list
    prop_idx = len(nodes)
    nodes.append(prop_node)
    
    rudder_idx = len(nodes)
    nodes.append(rudder_node)
    
    # Attach propeller and rudder as child nodes of the Hull node (index 0 / Part 17)
    hull_node = nodes[0]
    if "children" not in hull_node:
        hull_node["children"] = []
    hull_node["children"].extend([prop_idx, rudder_idx])
    
    print(f"Propeller Node registered at index: {prop_idx}")
    print(f"Rudder Node registered at index: {rudder_idx}")
    
    # Write sanitized file
    data["nodes"] = nodes
    with open(out_path, "w", encoding="utf-8") as f:
        json.dump(data, f, indent=2)
        
    print(f"\nSanitization complete. Saved clean glTF model to:\n  {out_path}")

if __name__ == "__main__":
    sanitize_and_articulate()
