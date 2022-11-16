# QEC-Playground

A research tool to explore Quantum Error Correction (QEC), primarily surface codes.

<strong style="color:red;">[Error] we're working on the documentation of this project, please wait for a formal release (1.0.0) before you want to use this project.</strong>

## Installation

See the [QEC-Playground Documentation: Installation](https://yuewuo.github.io/QEC-Playground/guide/installation.html) for the detailed instructions.
A brief example is below.

```bash
# Download the Blossom V Library [Optional]
wget -c https://pub.ist.ac.at/~vnk/software/blossom5-v2.05.src.tar.gz -O - | tar -xz
cp -r blossom5-v2.05.src/* backend/blossomV/
rm -r blossom5-v2.05.src

# Install the Python Dependencies [Optional]
sudo apt install python3 python3-pip
pip3 install networkx

# Install the Rust Toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.bashrc  # this will add `~/.cargo/bin` to path
cd backend/rust/
cargo build --release
cd ../../
```


## Command-line Interface

See the [QEC-Playground Documentation: CLI](https://yuewuo.github.io/QEC-Playground/guide/cli.html) for the detailed instructions.
A brief example use case is below.

Run `cargo run --release -- --help` under `backend/rust/` folder to get all provided commands of backend program.
The option `--help` prints out the information of this command, which can be helpful to find subcommands as well as to understand the purpose of each option.
An example output is below.

```init
QECPlayground 0.1.6
Yue Wu <yue.wu@yale.edu>, Namitha Liyanage (namitha.liyanage@yale.edu)
Quantum Error Correction Playground

USAGE:
    qecp <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    fpga_generator    fpga_generator
    help              Print this message or the help of the given subcommand(s)
    server            HTTP server for decoding information
    test              testing features
    tool              tools
```

To run a simulation to benchmark the logical error rate of decoder, run `cargo run --release -- tool benchmark --help`. An example output is below.

```bash
qecp-tool-benchmark 0.1.6
benchmark surface code decoders

USAGE:
    qecp tool benchmark [OPTIONS] <dis> <nms> <ps>

ARGS:
    <dis>    [di1,di2,di3,...,din] code distance of vertical axis
    <nms>    [nm1,nm2,nm3,...,nmn] number of noisy measurement rounds, must have exactly the
             same length as `dis`; note that a perfect measurement is always capped at the end,
             so to simulate a single round of perfect measurement you should set this to 0
    <ps>     [p1,p2,p3,...,pm] p = px + py + pz unless error model has special interpretation of
             this value

OPTIONS:
        --bias_eta <bias_eta>
            bias_eta = pz / (px + py) and px = py, px + py + pz = p. default to 1/2, which means px
            = pz = py [default: 0.5]
        ......
```

For example, to test code-distance-3 standard CSS surface code with depolarizing physical error rates 3%, 2% and 1% only on data qubits (i.e. perfect stabilizer measurements) using the default decoder (MWPM decoder), run:

```bash
cargo run --release -- tool benchmark [3] [0] [3e-2,2e-2,1e-2]
```

An example result is below.

```init
format: <p> <di> <nm> <total_repeats> <qec_failed> <error_rate> <dj> <confidence_interval_95_percent> <pe>
0.03 3 0 567712 10000 0.01761456513161603 3 1.9e-2 0
0.02 3 0 1255440 10000 0.007965334862677627 3 2.0e-2 0
0.01 3 0 4705331 10000 0.002125248999485902 3 2.0e-2 0
```


## Change Log

See [CHANGELOG.md](CHANGELOG.md)

## Contributions

Yue Wu (yue.wu@yale.edu): implement 3D GUI. design and implement interactive tutorial. propose and implement naïve decoder. implement MWPM decoder. Implement different variants of surface code and different decoders (see change log 2020.11.8 - 2022.3.20). The major developer and maintainer of this repository.

Guojun Chen: collaborator of CPSC 559 course project: design GUI. design and implement machine learning based weight optimized MWPM decoder.

Namitha Godawatte Liyanage: implement approximate MWPM decoder and FPGA related functionalities.

Neil He: bind library to Python.

## Attribution

When using QEC-Playground for research, please cite:

```
TODO: arXiv link for related papers (probably the fusion blossom paper)
```
