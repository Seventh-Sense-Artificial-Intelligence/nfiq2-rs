## Usage (Python)

Here's a simple example of how to use the NFIQ2 Python bindings:

```python
import nfiq2

# Create an NFIQ2 instance
nfiq2_instance: nfiq2.Nfiq2 = nfiq2.create_nfiq2()

for i in range(1, 4):
    image_bytes = open(f"../../../test_data/p1/p1_{i}.png", "rb").read()

    # Nfiq2Result has three fields: actionable, features and score. 
    # actionable: List of Nfiq2Value
    # features: List of Nfiq2Value
    # score: int (which is to be used for determing the quality of the fingerprint)
    # Nfiq2Value has two fields: value and name
    nfiq2_result: nfiq2.Nfiq2Result = nfiq2_instance.compute(image_bytes)

    # Get the score, for fingerprints this value should be high.
    print(nfiq2_result.score)
    assert nfiq2_result.score > 50

for name in ["face.jpeg", "landscape.jpg"]:
    image_bytes = open(f"../../../test_data/negative/{name}", "rb").read()
    nfiq2_result: nfiq2.Nfiq2Result = nfiq2_instance.compute(image_bytes)
    # Get the score, for non-fingerprints this value should be low.
    print(nfiq2_result.score) 
    assert nfiq2_result.score < 40
```

### Pytest for the python wheel of NFIQ2 Library

### Setup
First generate the python wheels by running the following command in the root directory:

```bash
./build_python.sh
```

This will create a virtual environment `.venv` and the python wheel in the `dist` folder.

Next, in the root directory, activate the virtual environment and install the wheel. Modify the wheel name if necessary:

```bash
source ./.venv/bin/activate
pip install ./dist/nfiq2_py-0.1.2-py3-none-manylinux_2_38_x86_64.whl
```

Next, change the directory to the test folder:
```bash
cd bindings/python/tests
```

### Run the pytests

Install the pytest and other dependencies as:
```bash
pip install -r requirements.txt
```

And run the test:
```bash
pytest test_nfiq2.py 
```