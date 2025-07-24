## NFIQ2-rs

NFIQ2-rs is a Rust library providing bindings for NIST's [NFIQ2](https://www.nist.gov/services-resources/software/nfiq-2) fingerprint quality assessment tool, using UniFFI for cross-language interoperability. It allows developers to integrate NFIQ2 functionality into their applications with ease.

HISTORY
=======

The NFIQ 2 algorithm was first released in April 2016 by NIST, in collaboration with Germany's Federal Office for Information Security and Federal Criminal Police Office, as well as research and development entities MITRE, Fraunhofer IGD, Hochschule Darmstadt, and Secunet.

This revision of the NFIQ 2 command line interface was first released by NIST in August 2020 as NFIQ 2 v2.1.

NFIQ 2 v2.3 was released in 2024 to align with the second edition of ISO/IEC 29794-4:2024.

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