## NFIQ2-rs

NFIQ2-rs is a Rust library providing bindings for NIST's [NFIQ2](https://www.nist.gov/services-resources/software/nfiq-2) fingerprint quality assessment tool, using UniFFI for cross-language interoperability. It allows developers to integrate NFIQ2 functionality into their applications with ease.

## History

The NFIQ 2 algorithm was first released in April 2016 by NIST, in collaboration with Germany's Federal Office for Information Security and Federal Criminal Police Office, as well as research and development entities MITRE, Fraunhofer IGD, Hochschule Darmstadt, and Secunet.

NFIQ 2 v2.3 was released in 2024 to align with the second edition of ISO/IEC 29794-4:2024.

## Features

 - Bindings to NFIQ2 functionality to get the overall quality score of a fingerprint image, as well as detailed quality metrics.

## Installation (Rust)

To use NFIQ2-rs in your Rust project, add the following to your `Cargo.toml`:

```toml
[dependencies]
nfiq2 = "0.1.0"
```

Then, you can use the library in your code:

```rust
use nfiq2::create_nfiq2;
fn main()-> Result<(), Box<dyn std::error::Error>> {
    let nfiq2 = create_nfiq2()?;
    let image_bytes = std::fs::read("test_data/p1/p1_1.png")?;
    let result = nfiq2.compute(&image_bytes)?;
    println!("Fingerprint quality score: {:?}", result.score);
    println!("Actionable metrics:");
    for metric in result.actionable {
        println!("{}: {}", metric.name, metric.value);
    }
    println!("Full features:");
    for feature in result.features {
        println!("{}: {}", feature.name, feature.value);
    }
    Ok(())
}
```

## Installation (Python)
```bash
pip install nfiq2-py
```

Then, you can use the library in your Python code:

```python
from nfiq2 import nfiq2

def main():
    nfiq2 = nfiq2.create_nfiq2()
    with open("test_data/p1/p1_1.png", "rb") as f:
        image_bytes = f.read()
    result = nfiq2.compute(image_bytes)
    print(f"Fingerprint quality score: {result.score}")
    print("Actionable metrics:")
    for metric in result.actionable:
        print(f"{metric.name}: {metric.value}")
    print("Full features:")
    for feature in result.features:
        print(f"{feature.name}: {feature.value}")

if __name__ == "__main__":
    main()
```

## Contributing

Contributions are welcome! Please open an issue or submit a pull request on GitHub.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

