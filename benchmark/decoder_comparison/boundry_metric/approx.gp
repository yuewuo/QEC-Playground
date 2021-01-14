set terminal postscript eps color "Arial, 28"
set xlabel "Depolarizing Error Rate (p)" font "Arial, 28"
set ylabel "Logical Error Rate (p_L)" font "Arial, 28"
set grid ytics
set size 1,1

# data generating commands:
# RUST_BACKTRACE=full cargo run --release -- tool fault_tolerant_benchmark [3] [3] [1e-1,5e-2,2e-2,1e-2,5e-3,2e-3,1e-3,5e-4,2e-4,1e-4,5e-5,2e-5,1e-5] -p0 -m100000000 -afalse
# RUST_BACKTRACE=full cargo run --release -- tool fault_tolerant_benchmark [5] [5] [1e-1,5e-2,2e-2,1e-2,5e-3,2e-3,1e-3,5e-4,2e-4,1e-4,5e-5,2e-5] -p0 -m100000000 -afalse
# RUST_BACKTRACE=full cargo run --release -- tool fault_tolerant_benchmark [7] [7] [1e-1,5e-2,2e-2,1e-2,5e-3,2e-3,1e-3,5e-4,2e-4] -p0 -m100000000 -b10 -e1000 -afalse

# RUST_BACKTRACE=full cargo run --release -- tool fault_tolerant_benchmark [9] [9] [1e-1,5e-2,2e-2,1e-2,5e-3,2e-3] -p0 -m100000000 -b1 -e1000 -afalse
# RUST_BACKTRACE=full cargo run --release -- tool fault_tolerant_benchmark [9] [9] [1e-3] -p0 -m100000000 -b1 -e200 -afalse
# RUST_BACKTRACE=full cargo run --release -- tool fault_tolerant_benchmark [9] [9] [5e-4,2e-4] -p0 -m100000000 -b1 -e10 -afalse

# RUST_BACKTRACE=full cargo run --release -- tool fault_tolerant_benchmark [11] [11] [1e-1,5e-2,2e-2,1e-2,5e-3,2e-3,1e-3] -p0 -m100000000 -b1 -e200 -afalse
# RUST_BACKTRACE=full cargo run --release -- tool fault_tolerant_benchmark [11] [11] [5e-4] -p0 -m100000000 -b1 -e20 -afalse

# RUST_BACKTRACE=full cargo run --release -- tool fault_tolerant_benchmark [13] [13] [1e-1,5e-2,2e-2,1e-2,5e-3,2e-3] -p0 -m100000000 -b1 -e200 -afalse
# RUST_BACKTRACE=full cargo run --release -- tool fault_tolerant_benchmark [13] [13] [1e-3] -p0 -m100000000 -b1 -e50 -afalse
# RUST_BACKTRACE=full cargo run --release -- tool fault_tolerant_benchmark [13] [13] [5e-4] -p0 -m100000000 -b1 -e10 -afalse

# collecting more data around the threshold
# RUST_BACKTRACE=full cargo run --release -- tool fault_tolerant_benchmark [3,5,7,9,11,13] [3,5,7,9,11,13] [3e-3] -p0 -m100000000 -b1 -e1000  -afalse
# RUST_BACKTRACE=full cargo run --release -- tool fault_tolerant_benchmark [3,5,7,9,11,13] [3,5,7,9,11,13] [2.4e-3] -p0 -m100000000 -b1 -e1000  -afalse
# RUST_BACKTRACE=full cargo run --release -- tool fault_tolerant_benchmark [3,5,7,9,11,13] [3,5,7,9,11,13] [2.3e-3] -p0 -m100000000 -b1 -e1000  -afalse

set logscale x
set xrange [0.00001:0.1]
set xtics ("10^{-5}" 0.00001, "10^{-4}" 0.0001, "10^{-3}" 0.001, "10^{-2}" 0.01, "10^{-1}" 0.1)
set logscale y
set ytics ("10^{-8}" 0.00000001, "10^{-7}" 0.0000001, "10^{-6}" 0.000001, "10^{-5}" 0.00001, "10^{-4}" 0.0001, "10^{-3}" 0.001, "10^{-2}" 0.01, "10^{-1}" 0.1)
set yrange [0.00000001:1]
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder

set output "equally_weighted.eps"

plot "d_3_3.txt" using 1:8 with linespoints lt rgb "red" linewidth 5 pointtype 6 pointsize 1.5 title "d = 3",\
    "d_5_5.txt" using 1:8 with linespoints lt rgb "blue" linewidth 5 pointtype 2 pointsize 1.5 title "d = 5",\
    "d_7_7.txt" using 1:8 with linespoints lt rgb "green" linewidth 5 pointtype 2 pointsize 1.5 title "d = 7",\
    "d_9_9.txt" using 1:8 with linespoints lt rgb "yellow" linewidth 5 pointtype 2 pointsize 1.5 title "d = 9",\
    "d_11_11.txt" using 1:8 with linespoints lt rgb "purple" linewidth 5 pointtype 2 pointsize 1.5 title "d = 11",\
    "d_13_13.txt" using 1:8 with linespoints lt rgb "orange" linewidth 5 pointtype 2 pointsize 1.5 title "d = 13"

set output '|ps2pdf -dEPSCrop equally_weighted.eps approx.pdf'
replot

set size 1,0.75
set output "equally_weighted_w.eps"
replot
set output '|ps2pdf -dEPSCrop equally_weighted_w.eps approx_w.pdf'
replot

set size 1,0.6
set output "equally_weighted_w_w.eps"
replot
set output '|ps2pdf -dEPSCrop equally_weighted_w_w.eps approx_w_w.pdf'
replot
