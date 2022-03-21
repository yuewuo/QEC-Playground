# QEC-Playground
An educational tool to the introduction of Quantum Error Correction (QEC)

## Installation

The code is already running at [https://wuyue98.cn/QECPlayground/](https://wuyue98.cn/QECPlayground/), but if you're interested in building them on yourself, follow this guide.

We assume Ubuntu 18.04 system, but installation on Win10 is feasible in similar way.

### Backend

We use Rust programming language for backend server, implementing decoder algorithms and serving as HTTP service. First install Rust and its package manager Cargo

```bash
curl https://sh.rustup.rs -sSf | sh
source ~/.bashrc  # this will add `~/.cargo/bin` to path
```

Use default installation and it will show up `Rust is installed now. Great!`

Then compile the backend

```bash
cd backend/rust/
cargo build --release
cargo run --release -- server  # start http server at http://127.0.0.1:8066
```

Install necessary python packets

```bash
sudo apt install python3 python3-pip
pip3 install networkx
```

### Frontend

We use Vue.js to implement code and use npm package manager to build code. Download and install npm from https://nodejs.org/en/download/ first, then install vue CLI

```bash
npm install -g @vue/cli
vue --version  # check whether installation is successful
vue ui  # this will start a web page
# select the folder of /qecplayground which contains the Vue project
# then you can use GUI in browser to build the code
```

Or you can use command line to build the frontend project

```bash
npm install
npm run build  # build code into dist/ folder
npm run serve  # fast debugging, hot re-compile
```

## Decoder Benchmark

Run `cargo run -- help ` under `rust/` folder to get all provided commands of backend program, the output is below:

```init
QECPlayground 1.0
Yue Wu yue.wu@yale.edu
Quantum Error Correction Playground for BIM'20 course

USAGE:
    rust_qecp <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    help      Prints this message or the help of the given subcommand(s)
    server    HTTP server for decoding information
    test      testing features
    tool      tools
```

To run a simulation to get the error rate of decoder, run `cargo run -- tool automatic_benchmark -h`

```bash
rust_qecp-tool-automatic_benchmark
automatically run benchmark with round upper bound, lower bound and minimum error cases

USAGE:
    rust_qecp tool automatic_benchmark [OPTIONS] <Ls> <ps>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -m, --max_N <max_N>                        maximum total count, default to 100000000
    -e, --min_error_cases <min_error_cases>    minimum error cases, default to 1000
    -q, --qec_decoder <qec_decoder>            available decoders, e.g. `naive_decoder`

ARGS:
    <Ls>    [L1,L2,L3,...,Ln]
    <ps>    [p1,p2,p3,...,pm]
```

You can use a subset of the parameters. For example, to test code distance 3, physical error rate 3e-2, 1e-2, 3e-3 using the `naive_decoder`, run:

```bash
cargo run --release -- tool automatic_benchmark [3] [3e-2,1e-2,3e-3] -q naive_decoder
```

Detailed commands to plot the graphs is in the comments of `/benchmark/*/*.gp`, for example we test the performance of MWPM decoder using

```bash
cargo run --release -- tool automatic_benchmark [3] [5e-1,2e-1,1e-1,5e-2,2e-2,1e-2,5e-3,2e-3,1e-3,5e-4,2e-4,1e-4,5e-5] -q maximum_max_weight_matching_decoder
cargo run --release -- tool automatic_benchmark [5] [5e-1,2e-1,1e-1,5e-2,2e-2,1e-2,5e-3,2e-3,1e-3,5e-4,2e-4] -q maximum_max_weight_matching_decoder
cargo run --release -- tool automatic_benchmark [7] [5e-1,2e-1,1e-1,5e-2,2e-2,1e-2,5e-3,2e-3,1e-3,5e-4] -q maximum_max_weight_matching_decoder
cargo run --release -- tool automatic_benchmark [9] [5e-1,2e-1,1e-1,5e-2,2e-2,1e-2,5e-3,2e-3,1e-3,5e-4] -q maximum_max_weight_matching_decoder
cargo run --release -- tool automatic_benchmark [11] [5e-1,2e-1,1e-1,5e-2,2e-2,1e-2,5e-3,2e-3,1e-3,5e-4] -q maximum_max_weight_matching_decoder
cargo run --release -- tool automatic_benchmark [13] [5e-1,2e-1,1e-1,5e-2,2e-2,1e-2,5e-3,2e-3] -q maximum_max_weight_matching_decoder
cargo run --release -- tool automatic_benchmark [15] [5e-1,2e-1,1e-1,5e-2,2e-2,1e-2,5e-3,2e-3] -q maximum_max_weight_matching_decoder
cargo run --release -- tool automatic_benchmark [25] [5e-1,2e-1,1e-1,5e-2,2e-2,1e-2,5e-3,2e-3] -q maximum_max_weight_matching_decoder -m 1000000
```

To maximize running speed, run

```bash
RUSTFLAGS="-C target-cpu=native" cargo run --release
```

## Change Log

See [CHANGELOG.md](CHANGELOG.md)

## Contributions

Yue Wu (yue.wu@yale.edu): implement 3D GUI. design and implement interactive tutorial. propose and implement naïve decoder. implement MWPM decoder. Implement different variants of surface code and different decoders (see change log 2020.11.8 - 2022.3.20). The major developer and maintainer of this repository.

Guojun Chen: collaborator of CPSC 559 course project: design GUI. design and implement machine learning based weight optimized MWPM decoder.

Namitha Godawatte Liyanage: implement approximate MWPM decoder and FPGA related functionalities.

