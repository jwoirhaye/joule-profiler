import json
import glob
import os
import matplotlib.pyplot as plt

RESULT_DIR = "bench_results"

points = []

for path in glob.glob(os.path.join(RESULT_DIR, "poll_*.json")):
    with open(path) as f:
        data = json.load(f)

    f_req = int(os.path.basename(path).split("_")[1].split(".")[0])
    duration_s = data["duration_ms"] / 1000000.0
    measure_count = data["measure_count"]

    if duration_s <= 0 or measure_count <= 0:
        continue

    f_real = measure_count / duration_s

    points.append((f_req, f_real))

# TRI NUMÉRIQUE PAR f_req
points.sort(key=lambda x: x[0])

f_req = [p[0] for p in points]
f_real = [p[1] for p in points]

# Graphe demandé vs réel
plt.figure()
plt.plot(f_req, f_real, marker="o")
plt.xlabel("Requested polling frequency (Hz)")
plt.ylabel("Real polling frequency (Hz)")
plt.title("RAPL polling: requested vs real frequency")
plt.grid(True)
plt.show()