#!/bin/bash
#SBATCH -J rotated-planar-code-capacity-hyperUF
#SBATCH -p day
#SBATCH -t 23:00:00
#SBATCH --cpus-per-task=4
#SBATCH --mail-type=ALL
#SBATCH --mem=16G

module load miniconda
module load GCC

conda activate par-mwpf

source $HOME/.bashrc
rustup default nightly

export TMPDIR=/tmp
export CARGO_BUILD_JOBS=1

export RUSTFLAGS="-C target-cpu=native"
export CFLAGS="-gdwarf-2"

# Use environment variables (set by sbatch --export)
FEATURES="${FEATURES:-hyperion mwpf/unsafe_pointer}"
CUSTOMIZE_FILENAME="${CUSTOMIZE_FILENAME:-unsafe}"

python run.py --features "$FEATURES" --customize-filename "$CUSTOMIZE_FILENAME"