set terminal postscript eps color "Arial, 28"
set xlabel "Code Distance (d), with d^2+(d-1)^2 data qubits" font "Arial, 28"
set ylabel "Logical Error Rate (p_L)" font "Arial, 28"
set grid ytics
set size 1,1

# data generating commands:

# p0.02_css   p0.02_xzzx
# RUST_BACKTRACE=full cargo run --release -- tool fault_tolerant_benchmark [3,5,7,9,11,13,15,17,19] [0,0,0,0,0,0,0,0,0] [0.02] -p0-m100000000 --use_xzzx_code --shallow_error_on_bottom -e1000000 --bias_eta 10
# RUST_BACKTRACE=full cargo run --release -- tool fault_tolerant_benchmark [3,5,7,9,11,13,15,17,19] [0,0,0,0,0,0,0,0,0] [0.02] -p0-m100000000 --shallow_error_on_bottom -e1000000 --bias_eta 10

# p0.01_css   p0.01_xzzx
# RUST_BACKTRACE=full cargo run --release -- tool fault_tolerant_benchmark [3,5,7,9,11,13,15,17,19] [0,0,0,0,0,0,0,0,0] [0.01] -p0-m100000000 --use_xzzx_code --shallow_error_on_bottom -e1000000 --bias_eta 10
# RUST_BACKTRACE=full cargo run --release -- tool fault_tolerant_benchmark [3,5,7,9,11,13,15,17,19] [0,0,0,0,0,0,0,0,0] [0.01] -p0-m100000000 --shallow_error_on_bottom -e1000000 --bias_eta 10

set xrange [3:19]
set xtics ("3" 3, "5" 5, "7" 7, "9" 9, "11" 11, "13" 13, "15" 15, "17" 17, "19" 19)
set logscale y
set ytics ("10^{-8}" 0.00000001, "10^{-7}" 0.0000001, "10^{-6}" 0.000001, "10^{-5}" 0.00001, "10^{-4}" 0.0001, "10^{-3}" 0.001, "10^{-2}" 0.01, "10^{-1}" 0.1)
set yrange [0.00000001:0.1]
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder

set output "fixed_noise_compare_XZZX_and_CCS_eta10_small_p.eps"

plot "css_p0.02.txt" using 2:6 with linespoints lt rgb "red" linewidth 5 pointtype 6 pointsize 1.5 title "CSS Surface Code p = 0.02",\
    "xzzx_p0.02.txt" using 2:6 with linespoints lt rgb "blue" linewidth 5 pointtype 2 pointsize 1.5 title "XZZX Surface Code p = 0.02",\
    "css_p0.01.txt" using 2:6 with linespoints lt rgb "green" linewidth 5 pointtype 6 pointsize 1.5 title "CSS Surface Code p = 0.01",\
    "xzzx_p0.01.txt" using 2:6 with linespoints lt rgb "yellow" linewidth 5 pointtype 2 pointsize 1.5 title "XZZX Surface Code p = 0.01"

system("ps2pdf -dEPSCrop fixed_noise_compare_XZZX_and_CCS_eta10_small_p.eps fixed_noise_compare_XZZX_and_CCS_eta10_small_p.pdf")

set size 1,0.75
set output "fixed_noise_compare_XZZX_and_CCS_eta10_small_p_w.eps"
replot
system("ps2pdf -dEPSCrop fixed_noise_compare_XZZX_and_CCS_eta10_small_p_w.eps fixed_noise_compare_XZZX_and_CCS_eta10_small_p_w.pdf")

set size 1,0.6
set output "fixed_noise_compare_XZZX_and_CCS_eta10_small_p_w_w.eps"
replot
system("ps2pdf -dEPSCrop fixed_noise_compare_XZZX_and_CCS_eta10_small_p_w_w.eps fixed_noise_compare_XZZX_and_CCS_eta10_small_p_w_w.pdf")
