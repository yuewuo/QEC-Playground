# data generating commands:
cd ../../backend/rust
RUST_BACKTRACE=full cargo run --release -- tool decoder_comparison_benchmark [3] [3] [1e-1,5e-2,2e-2,1e-2,5e-3,2e-3,1e-3,5e-4,2e-4,1e-4,5e-5,2e-5,1e-5] -p0 -m100000000 --mini_batch 1000
RUST_BACKTRACE=full cargo run --release -- tool decoder_comparison_benchmark [5] [5] [1e-1,5e-2,2e-2,1e-2,5e-3,2e-3,1e-3,5e-4,2e-4,1e-4,5e-5,2e-5] -p0 -m100000000 --mini_batch 1000
RUST_BACKTRACE=full cargo run --release -- tool decoder_comparison_benchmark [7] [7] [1e-1,5e-2,2e-2,1e-2,5e-3,2e-3,1e-3,5e-4,2e-4] -p0 -m100000000 -e1000 --mini_batch 10

RUST_BACKTRACE=full cargo run --release -- tool decoder_comparison_benchmark [9] [9] [1e-1,5e-2,2e-2,1e-2,5e-3,2e-3] -p0 -m100000000 -b1 -e1000 
RUST_BACKTRACE=full cargo run --release -- tool decoder_comparison_benchmark [9] [9] [1e-3] -p0 -m100000000 -b1 -e200 
RUST_BACKTRACE=full cargo run --release -- tool  decoder_comparison_benchmark [9] [9] [5e-4,2e-4] -p0 -m100000000 -b1 -e10 

RUST_BACKTRACE=full cargo run --release -- tool decoder_comparison_benchmark [11] [11] [1e-1,5e-2,2e-2,1e-2,5e-3,2e-3,1e-3] -p0 -m100000000 -b1 -e200
RUST_BACKTRACE=full cargo run --release -- tool decoder_comparison_benchmark [11] [11] [5e-4] -p0 -m100000000 -b1 -e20  

RUST_BACKTRACE=full cargo run --release -- tool decoder_comparison_benchmark [13] [13] [1e-1,5e-2,2e-2,1e-2,5e-3,2e-3] -p0 -m100000000 -b1 -e200 
RUST_BACKTRACE=full cargo run --release -- tool decoder_comparison_benchmark [13] [13] [1e-3] -p0 -m100000000 -b1 -e50  
RUST_BACKTRACE=full cargo run --release -- tool decoder_comparison_benchmark [13] [13] [5e-4] -p0 -m100000000 -b1 -e10 