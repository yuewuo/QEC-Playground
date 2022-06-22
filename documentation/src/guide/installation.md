# Installation

There are multiple ways to install this library.
Choose any one of the methods below that best suit your needs.

## Pre-compiled Binaries

<strong style="color:red;">TODO: upload the binary and alert the limitation of Blossom V library license issue</strong>

## Build from Source using Rust

This is the recommended way to install the library for two reasons.
First, the pre-compiled binary doesn't include the Blossom V library due to incompatible license, and thus is not capable of running any decoder that based on Minimum-Weight Perfect Matching (MWPM) algorithm.
Second, you can access all the examples as well as the simulation scripts and data in our published papers only from the source code repository.

### Download the Blossom V Library [Optional]

In order to use the MWPM algorithm (e.g. in the MWPM decoder or the tailored-SC decoder), you need to download the Blossom V @@kolmogorov2009blossom library from [https://pub.ist.ac.at/~vnk/software.html](https://pub.ist.ac.at/~vnk/software.html) into the folder `backend/blossomV`.
You're responsible for requesting a proper license for the use of this library, as well as obeying any requirement.

An example of downloading the Blossom V library is below. Note that the latest version number and website address may change over time.

```bash
wget -c https://pub.ist.ac.at/~vnk/software/blossom5-v2.05.src.tar.gz -O - | tar -xz
cp -r blossom5-v2.05.src/* backend/blossomV/
rm -r blossom5-v2.05.src
```

You don't need to compile the Blossom V library manually.

Note that you can still compile the project without the Blossom V library.
The build script automatically detects whether the Blossom V library exists and enables the feature accordingly.

### Install the Python Dependencies [Optional]

To enable the binding between Rust and Python, install the following packages

```bash
sudo apt install python3 python3-pip
pip3 install networkx
```

### Install the Rust Toolchain

We need the Rust toolchain to compile the project written in the Rust programming language.
Please see [https://rustup.rs/](https://rustup.rs/) for the latest instructions.
An example on Unix-like operating systems is below.

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.bashrc  # this will add `~/.cargo/bin` to path
```

After installing the Rust toolchain successfully, you can compile the backend by

```bash
cd backend/rust/
cargo build --release
```

### Install Frontend tools [Optional]

<strong style="color:red;">TODO: We plan to remove the dependency of Node.js for building the frontend, by rewriting the tutorials and visualization pages as single-page Vue applications.</strong>

The code is already running at [https://wuyue98.cn/QECPlayground/](https://wuyue98.cn/QECPlayground/), but if you're interested in building them on yourself, follow this guide.

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
cd qecplayground
npm install
npm run build  # build code into dist/ folder
npm run serve  # fast debugging, hot re-compile
```

### Install mdbook to build the documentation

In order to build this documentation, you need to install [mdbook](https://crates.io/crates/mdbook) and several plugins.

```bash
cargo install mdbook
cargo install mdbook-bib
cd documentation
mdbook serve  # dev mode, automatically refresh the local web page on code change
mdbook build  # build deployment in /docs folder, to be recognized by GitHub Pages
```
