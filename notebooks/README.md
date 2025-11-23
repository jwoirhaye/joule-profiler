# joule-profiler Notebooks

Jupyter notebooks for analyzing and visualizing energy consumption data from joule-profiler.

## 📚 Available Notebooks

- **`quickstart.ipynb`** - Simple mode analysis
    - Single measurement visualization
    - Multiple iterations comparison
    - Energy trends and variance analysis
    - JSON and CSV support

- **`phases.ipynb`** - Phases mode analysis
    - Phase-by-phase energy breakdown
    - Pre-work, work, and post-work analysis
    - Multi-iteration phase comparison
    - Per-domain visualizations


## Setup

### 1. Create a virtual environment

```bash
# Create venv
python -m venv venv

# Activate venv
source venv/bin/activate
```

### 2. Install dependencies

```bash
pip install -r requirements.txt
```

### 3. Launch Jupyter

```bash
jupyter notebook
```

Your browser will open with the Jupyter interface. Click on quickstart.ipynb or phases.ipynb to start.

## 📊 Example Data

Example datasets are provided in `../examples/data/`:

- `simple.json` / `simple.csv` - Single measurement
- `simple-iterations.json` / `simple-iterations.csv` - 5 iterations
- `phases.json` / `phases.csv` - Phase detection
- `phases-iterations.json` / `phases-iterations.csv` - Phases with 5 iterations

## 🔬 Generate Your Own Data

### Simple Mode

```bash

# JSON format
sudo joule-profiler simple --json --jouleit-file my-data.json -- python3 ../examples/programs/workload.py

# CSV format
sudo joule-profiler simple --csv --jouleit-file my-data.csv -- python3 ../examples/programs/workload.py

# Multiple iterations
sudo joule-profiler simple --json --jouleit-file my-iterations.json -n 5 -- python3 ../examples/programs/workload.py
```

### Phases Mode
```bash

# JSON format
sudo joule-profiler phases --json --jouleit-file my-phases.json -- python3 ../examples/programs/workload.py

# CSV format
sudo joule-profiler phases --csv --jouleit-file my-phases.csv -- python3 ../examples/programs/workload.py

# Multiple iterations
sudo joule-profiler phases --json --jouleit-file my-phases-iterations.json -n 5 -- python3 ../examples/programs/workload.py
```

Then load your data in the notebooks by changing the file path:
```python

# Instead of
data = pd.read_json('../examples/data/simple.json', typ='series')

# Use
data = pd.read_json('my-data.json', typ='series')
```