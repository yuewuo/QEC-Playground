set terminal postscript eps color "Arial, 28"
set xlabel "Code Distance (d)" font "Arial, 28"
set ylabel "Logical Error Rate (p_L)" font "Arial, 28"
set grid ytics
set size 1,1

# data generating commands:
# cargo run --release -- tool distributed_union_find_decoder_standard_planar_benchmark [3,5,7,9,11,13] [1e-2] -p0 -b100 -m100000000 -e100000000 --only_count_logical_x --output_cycle_distribution
# cargo run --release -- tool distributed_union_find_decoder_standard_planar_benchmark [15,17,19] [1e-2] -p0 -b100 -m10000000000 -e100 --only_count_logical_x --output_cycle_distribution

set logscale y
set ytics ("10^{-8}" 0.00000001, "10^{-7}" 0.0000001, "10^{-6}" 0.000001, "10^{-5}" 0.00001, "10^{-4}" 0.0001, "10^{-3}" 0.001, "10^{-2}" 0.01, "10^{-1}" 0.1)
set yrange [0.00000001:1]
set key outside horizontal top center font "Arial, 24"

set output "fixed_error_rate.eps"

plot "results.txt" using 2:5 with linespoints lt rgb "red" linewidth 5 pointtype 6 pointsize 1.5 title "p = 0.01"

set output '|ps2pdf -dEPSCrop fixed_error_rate.eps fixed_error_rate.pdf'
replot

set size 1,0.75
set output "fixed_error_rate_w.eps"
replot
set output '|ps2pdf -dEPSCrop fixed_error_rate_w.eps fixed_error_rate_w.pdf'
replot

set size 1,0.6
set output "fixed_error_rate_w_w.eps"
replot
set output '|ps2pdf -dEPSCrop fixed_error_rate_w_w.eps fixed_error_rate_w_w.pdf'
replot
