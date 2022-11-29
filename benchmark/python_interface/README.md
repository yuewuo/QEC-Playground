# Benchmark Python Interface

It provides an interface to assign noise model with Python and run decoding benchmark without having to modify Rust code. By using this interface, you first provide a set of supported command-line configurations, then the library will fetch the possible error positions from Rust program. After modifying noise model accordingly, you can choose to save a temporary file or save to a specific location of this description of noise model.

Example code at `example.py`

## API

