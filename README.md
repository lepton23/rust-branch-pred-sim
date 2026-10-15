# Branch predictor sim usage guide

Build Instructions: \
`cargo build --release`

Run instructions: \
`/target/release/p2-bp <trace-file-path> <bp-alg> <table-size optional> 2>/dev/null`

Measurement instructions: \
- `source .venv/bin/activate`
- `pip install -r requirements.txt`
- `python measure.py`
- `python analyse.py`

### NB: Will not work without properly formed trace file as was developed for uni project
