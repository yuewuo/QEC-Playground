#!/bin/bash

# Master script to run all benchmarks with different features
# Run from QEC-Playground/benchmark/paper_hyperion/accuracy/

set -e  # Exit on error

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Define benchmark configurations: (features_string, customize_filename)
BENCHMARK_CONFIGS=(
    "hyperion mwpf/unsafe_pointer:unsafe"
    "hyperion:safe"
)

echo "Starting all benchmark runs..."

# Get all subdirectories
for dir in "$SCRIPT_DIR"/*; do
    if [ -d "$dir" ]; then
        folder_name=$(basename "$dir")
        
        # Check if hyperion and hyperUF subdirectories exist
        if [ -d "$dir/hyperion" ] && [ -d "$dir/hyperUF" ]; then
            echo ""
            echo "=========================================="
            echo "Processing folder: $folder_name"
            echo "=========================================="
            
            # Iterate through each benchmark configuration
            for config in "${BENCHMARK_CONFIGS[@]}"; do
                IFS=':' read -r features customize_filename <<< "$config"
                
                # Run hyperion
                if [ -f "$dir/hyperion/run.py" ]; then
                    echo "Running $folder_name/hyperion with features: '$features', filename: '$customize_filename'"
                    cd "$dir/hyperion"
                    # Use environment variables instead of command-line arguments
                    sbatch --export=FEATURES="$features",CUSTOMIZE_FILENAME="$customize_filename" script.sh
                    cd "$SCRIPT_DIR"
                fi
                
                # Run hyperUF
                if [ -f "$dir/hyperUF/run.py" ]; then
                    echo "Running $folder_name/hyperUF with features: '$features', filename: '$customize_filename'"
                    cd "$dir/hyperUF"
                    sbatch --export=FEATURES="$features",CUSTOMIZE_FILENAME="$customize_filename" script.sh
                    cd "$SCRIPT_DIR"
                fi
            done
        fi
    fi
done

echo ""
echo "=========================================="
echo "All jobs submitted!"
echo "=========================================="