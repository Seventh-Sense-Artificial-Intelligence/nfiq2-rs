python3 -m venv .venv
source .venv/bin/activate
pip install maturin
maturin build --release --manifest-path Cargo.toml --out dist
./patch_maturin_wheel.sh