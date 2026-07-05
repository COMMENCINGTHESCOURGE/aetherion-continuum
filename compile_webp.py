import os
from PIL import Image

output_dir = r"C:\Users\dasha\.gemini\antigravity-ide\brain\43b6fe1c-0cbf-4636-9f3d-29dc0e11cb9c"
temp_frame_dir = os.path.join(output_dir, "temp_frames")

frame_files = [os.path.join(temp_frame_dir, f) for f in sorted(os.listdir(temp_frame_dir)) if f.endswith(".png")]
if frame_files:
    images = [Image.open(fp) for fp in frame_files]
    webp_path = os.path.join(output_dir, "new_data_turntable.webp")
    images[0].save(
        webp_path,
        save_all=True,
        append_images=images[1:],
        duration=33, # ~30 fps
        loop=0
    )
    print(f"Turntable WebP animation compiled successfully at: {webp_path}")
    
    # Cleanup temp files
    for fp in frame_files:
        try:
            os.remove(fp)
        except:
            pass
    try:
        os.rmdir(temp_frame_dir)
    except:
        pass
else:
    print("No frame files found to compile.")
