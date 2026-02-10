#!/bin/bash
#SBATCH -J qec-hyperion-rotated-planar-circuit-level
#SBATCH -p day
#SBATCH -t 3:00:00
#SBATCH --cpus-per-task=4
#SBATCH --mail-type=ALL
#SBATCH --mem=16G

module load miniconda
module load GCC

conda activate par-mwpf

source $HOME/.bashrc
rustup default nightly

# source $HOME/.cargo/env
export RUSTFLAGS="-C target-cpu=native"
export CFLAGS="-gdwarf-2"

# cargo build --release --features hyperion
python run.py