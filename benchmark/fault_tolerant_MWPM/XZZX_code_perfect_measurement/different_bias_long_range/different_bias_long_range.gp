set terminal postscript eps color "Arial, 28"
set xlabel "Depolarizing Error Rate (p)" font "Arial, 28"
set ylabel "Logical Error Rate (p_L)" font "Arial, 28"
set grid ytics
set size 1,1

# data generating commands:
# RUST_BACKTRACE=full cargo run --release -- tool fault_tolerant_benchmark [13] [0] [5e-1,3e-1,2e-1,1.5e-1,1e-1,5e-2,3e-2,2e-2] -p0 -b1000 -m100000000 --use_xzzx_code --shallow_error_on_bottom --bias_eta 0.5
# RUST_BACKTRACE=full cargo run --release -- tool fault_tolerant_benchmark [13] [0] [5e-1,3e-1,2e-1,1.5e-1,1e-1,5e-2,3e-2,2e-2] -p0 -b1000 -m100000000 --use_xzzx_code --shallow_error_on_bottom --bias_eta 1
# RUST_BACKTRACE=full cargo run --release -- tool fault_tolerant_benchmark [13] [0] [5e-1,3e-1,2e-1,1.5e-1,1e-1,5e-2,3e-2,2e-2] -p0 -b1000 -m100000000 --use_xzzx_code --shallow_error_on_bottom --bias_eta 10
# RUST_BACKTRACE=full cargo run --release -- tool fault_tolerant_benchmark [13] [0] [5e-1,3e-1,2e-1,1.5e-1,1e-1,5e-2,3e-2,2e-2] -p0 -b1000 -m100000000 --use_xzzx_code --shallow_error_on_bottom --bias_eta 100

set logscale x
set xrange [0.02:0.5]
set xtics ("0.02" 0.02, "0.05" 0.05, "0.1" 0.1, "0.2" 0.2, "0.5" 0.5)
set logscale y
set ytics ("10^{-8}" 0.00000001, "10^{-7}" 0.0000001, "10^{-6}" 0.000001, "10^{-5}" 0.00001, "10^{-4}" 0.0001, "10^{-3}" 0.001, "10^{-2}" 0.01, "10^{-1}" 0.1)
set yrange [0.00000001:1]
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder

set output "XZZX_code_perfect_measurement.eps"

plot "eta_0.5.txt" using 1:6 with linespoints lt rgb "red" linewidth 5 pointtype 6 pointsize 1.5 title "{/Symbol h} = 0.5",\
    "eta_1.txt" using 1:6 with linespoints lt rgb "blue" linewidth 5 pointtype 2 pointsize 1.5 title "{/Symbol h} = 1",\
    "eta_10.txt" using 1:6 with linespoints lt rgb "yellow" linewidth 5 pointtype 2 pointsize 1.5 title "{/Symbol h} = 10",\
    "eta_100.txt" using 1:6 with linespoints lt rgb "purple" linewidth 5 pointtype 2 pointsize 1.5 title "{/Symbol h} = 100"

set output '|ps2pdf -dEPSCrop XZZX_code_perfect_measurement.eps XZZX_code_perfect_measurement.pdf'
replot

set size 1,0.75
set output "XZZX_code_perfect_measurement_w.eps"
replot
set output '|ps2pdf -dEPSCrop XZZX_code_perfect_measurement_w.eps XZZX_code_perfect_measurement_w.pdf'
replot

set size 1,0.6
set output "XZZX_code_perfect_measurement_w_w.eps"
replot
set output '|ps2pdf -dEPSCrop XZZX_code_perfect_measurement_w_w.eps XZZX_code_perfect_measurement_w_w.pdf'
replot
