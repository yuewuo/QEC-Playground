set terminal postscript eps color "Arial, 28"
set xlabel "Code Distance (d), with d^2+(d-1)^2 data qubits" font "Arial, 28"
set ylabel "Logical Error Rate (p_L)" font "Arial, 28"
set grid ytics
set size 1,1

# data generating commands:

# p0.2_MWPM   p0.2_UF
# RUST_BACKTRACE=full cargo run --release -- tool fault_tolerant_benchmark [3,5,7,9,11,13,15,17,19] [0,0,0,0,0,0,0,0,0] [0.2] -p0-m100000000 --use_xzzx_code --shallow_error_on_bottom -e1000000 --bias_eta 0.5
# RUST_BACKTRACE=full cargo run --release -- tool fault_tolerant_benchmark [3,5,7,9,11,13,15,17,19] [0,0,0,0,0,0,0,0,0] [0.2] -p0-m100000000 --use_xzzx_code --shallow_error_on_bottom -e1000000 --bias_eta 0.5 --decoder UF --max_half_weight 10

# p0.1_MWPM   p0.1_UF
# RUST_BACKTRACE=full cargo run --release -- tool fault_tolerant_benchmark [3,5,7,9,11,13,15,17,19] [0,0,0,0,0,0,0,0,0] [0.1] -p0-m100000000 --use_xzzx_code --shallow_error_on_bottom -e1000000 --bias_eta 0.5
# RUST_BACKTRACE=full cargo run --release -- tool fault_tolerant_benchmark [3,5,7,9,11,13,15,17,19] [0,0,0,0,0,0,0,0,0] [0.1] -p0-m100000000 --use_xzzx_code --shallow_error_on_bottom -e1000000 --bias_eta 0.5 --decoder UF --max_half_weight 10

set xrange [3:19]
set xtics ("3" 3, "5" 5, "7" 7, "9" 9, "11" 11, "13" 13, "15" 15, "17" 17, "19" 19)
set logscale y
set ytics ("0.001" 0.001, "0.005" 0.005, "0.01" 0.01, "0.05" 0.05, "0.1" 0.1, "0.5" 0.5)
set yrange [0.001:0.5]
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder

set output "compare_with_MWPM_eta0.5.eps"

plot "uf_p0.2.txt" using 2:5 with linespoints lt rgb "red" linewidth 5 pointtype 6 pointsize 1.5 title "UnionFind Decoder p = 0.2",\
    "mwpm_p0.2.txt" using 2:6 with linespoints lt rgb "blue" linewidth 5 pointtype 2 pointsize 1.5 title "MWPM Decoder p = 0.2",\
    "uf_p0.1.txt" using 2:5 with linespoints lt rgb "green" linewidth 5 pointtype 6 pointsize 1.5 title "UnionFind Decoder p = 0.1",\
    "mwpm_p0.1.txt" using 2:6 with linespoints lt rgb "yellow" linewidth 5 pointtype 2 pointsize 1.5 title "MWPM Decoder p = 0.1"

system("ps2pdf -dEPSCrop compare_with_MWPM_eta0.5.eps compare_with_MWPM_eta0.5.pdf")

set size 1,0.75
set output "compare_with_MWPM_eta0.5_w.eps"
replot
system("ps2pdf -dEPSCrop compare_with_MWPM_eta0.5_w.eps compare_with_MWPM_eta0.5_w.pdf")

set size 1,0.6
set output "compare_with_MWPM_eta0.5_w_w.eps"
replot
system("ps2pdf -dEPSCrop compare_with_MWPM_eta0.5_w_w.eps compare_with_MWPM_eta0.5_w_w.pdf")
