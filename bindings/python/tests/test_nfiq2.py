import nfiq2
import pytest

@pytest.fixture(scope="module")
def nfiq2_instance():
    return nfiq2.create_nfiq2()

@pytest.mark.parametrize("filename,expected_min_score", [
    ("p1_1.png", 60),
    ("p1_2.png", 60),
    ("p1_3.png", 60),
])
def test_fingerprint_quality_positive(nfiq2_instance, filename, expected_min_score):
    with open(f"../../../test_data/p1/{filename}", "rb") as f:
        image_bytes = f.read()
    result = nfiq2_instance.compute(image_bytes)
    print(f"{filename} → Score: {result.score}")
    assert result.score >= expected_min_score, f"{filename} expected score >= {expected_min_score}"

@pytest.mark.parametrize("filename,expected_max_score", [
    ("face.jpeg", 40),
    ("landscape.jpg", 40),
])
def test_fingerprint_quality_negative(nfiq2_instance, filename, expected_max_score):
    with open(f"../../../test_data/negative/{filename}", "rb") as f:
        image_bytes = f.read()
    result = nfiq2_instance.compute(image_bytes)
    print(f"{filename} → Score: {result.score}")
    assert result.score <= expected_max_score, f"{filename} expected score <= {expected_max_score}"
