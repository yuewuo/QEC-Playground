# Introduction

<strong style="color:red;">[Error] we're working on the documentation of this project, please wait for a formal release (1.0.0) before you want to use this project.</strong>

<strong style="color:red;">TODO: add citations to all concepts</strong>

**QEC-Playground** is a command line tool and library to explore quantum error correction, primarily surface code.
It integrates the whole workflow from simulating surface code to decoding the syndrome and validating the decoding results.
Every component in this workflow can be customized independently, so that user can explore different codes or decoders with minimum effort.

On top of this library, we provide a Python binding so that you don't have to learn Rust programming language while enjoying the speed gain of Rust.
You can implement your own surface code, noise model or decoder in Python, while letting other components to run at full speed in Rust.

- **Simulator**
  - generates random qubit errors and simulates the measurement outcome (syndrome), given the Clifford gate implementation of a surface code
  - **Code Builder**
    - builds the Clifford gate implementation of a supported surface code, given a few parameters like code distances and how many rounds of measurements
    - <strong style="color:red;">TODO:</strong> *Python: full customization supported*
- **Noise Model**
  - describes the error distribution, supporting single-qubit and two-qubit correlated Pauli errors and erasure errors
  - **Noise Model Builder**
    - builds a supported noise model, given customized parameters like initialization error rate, measurement error rate, noise bias, etc.
    - <strong style="color:red;">TODO:</strong> *Python: full customization supported*
- **Decoder**
  - computes the correction pattern that tries to minimize the logical error rate of the logical qubit encoded in this surface code, given a measurement result
  - <strong style="color:red;">TODO:</strong> *Python: full customization supported*
  - **Model Graph**
    - automatically builds the model graph from the Clifford gate implementation of any surface code for graph-based decoders, e.g. MWPM decoder and UF decoder
    - <strong style="color:red;">TODO:</strong> *Python: manually change the model graph*
    - **Complete Mode Graph**
      - computes the complete model graph out of the model graph either offline (precompute the complete graph, memory-demanding) or online (compute edges on the fly, CPU-demanding)
      - <strong style="color:red;">TODO:</strong> *Python: manually change the complete model graph in the offline mode*
  - **Erasure Graph**
    - automatically builds the erasure graph from the Clifford gate implementation of any surface code for graph-based decoders, e.g. MWPM decoder and UF decoder

# Contributing

**QEC-Playground** is free and open source.
You can find the source code on [Github](https://github.com/yuewuo/QEC-Playground), where you can post issues and feature requests using [Github issue tracker](https://github.com/yuewuo/QEC-Playground/issues).
Currently the project is maintained by [Yue Wu](https://wuyue98.cn/) at [Yale Efficient Computing Lab](http://www.yecl.org/).
If you'd like to contribute to the project, consider [emailing the maintainer](mailto:yue.wu@yale.edu) or opening a [pull request](https://github.com/yuewuo/QEC-Playground/pulls).

# License

The QEC-Playground source and documentation are released under the [MIT License](https://opensource.org/licenses/MIT).
