set terminal postscript eps color "Arial, 28"
set xlabel "maximum half weight" font "Arial, 28"
set ylabel "Logical Error Rate (p_L)" font "Arial, 28"
set grid ytics
set size 1,1

# data generating commands:

# RUST_BACKTRACE=full cargo run --release -- tool fault_tolerant_benchmark [7] [0] [0.1,0.05,0.02] -p0-m100000000 -e10000 --use_xzzx_code --shallow_error_on_bottom --bias_eta 100000 --decoder UF --max_half_weight 1
# RUST_BACKTRACE=full cargo run --release -- tool fault_tolerant_benchmark [7] [0] [0.1,0.05,0.02] -p0-m100000000 -e10000 --use_xzzx_code --shallow_error_on_bottom --bias_eta 100000 --decoder UF --max_half_weight 2
# RUST_BACKTRACE=full cargo run --release -- tool fault_tolerant_benchmark [7] [0] [0.1,0.05,0.02] -p0-m100000000 -e10000 --use_xzzx_code --shallow_error_on_bottom --bias_eta 100000 --decoder UF --max_half_weight 3
# RUST_BACKTRACE=full cargo run --release -- tool fault_tolerant_benchmark [7] [0] [0.1,0.05,0.02] -p0-m100000000 -e10000 --use_xzzx_code --shallow_error_on_bottom --bias_eta 100000 --decoder UF --max_half_weight 4
# RUST_BACKTRACE=full cargo run --release -- tool fault_tolerant_benchmark [7] [0] [0.1,0.05,0.02] -p0-m100000000 -e10000 --use_xzzx_code --shallow_error_on_bottom --bias_eta 100000 --decoder UF --max_half_weight 5
# RUST_BACKTRACE=full cargo run --release -- tool fault_tolerant_benchmark [7] [0] [0.1,0.05,0.02] -p0-m100000000 -e10000 --use_xzzx_code --shallow_error_on_bottom --bias_eta 100000 --decoder UF --max_half_weight 6
# RUST_BACKTRACE=full cargo run --release -- tool fault_tolerant_benchmark [7] [0] [0.1,0.05,0.02] -p0-m100000000 -e10000 --use_xzzx_code --shallow_error_on_bottom --bias_eta 100000 --decoder UF --max_half_weight 7
# RUST_BACKTRACE=full cargo run --release -- tool fault_tolerant_benchmark [7] [0] [0.1,0.05,0.02] -p0-m100000000 -e10000 --use_xzzx_code --shallow_error_on_bottom --bias_eta 100000 --decoder UF --max_half_weight 8
# RUST_BACKTRACE=full cargo run --release -- tool fault_tolerant_benchmark [7] [0] [0.1,0.05,0.02] -p0-m100000000 -e10000 --use_xzzx_code --shallow_error_on_bottom --bias_eta 100000 --decoder UF --max_half_weight 9
# RUST_BACKTRACE=full cargo run --release -- tool fault_tolerant_benchmark [7] [0] [0.1,0.05,0.02] -p0-m100000000 -e10000 --use_xzzx_code --shallow_error_on_bottom --bias_eta 100000 --decoder UF --max_half_weight 10

# python -c "for i in range(1, 11): print('\'%d\' %d' % tuple([i for j in range(2)]), end=', ')"
set xrange [1:10]
set xtics ('1' 1, '2' 2, '3' 3, '4' 4, '5' 5, '6' 6, '7' 7, '8' 8, '9' 9, '10' 10)
set logscale y
set ytics ("10^{-5}" 0.00001, "10^{-4}" 0.0001, "10^{-3}" 0.001, "10^{-2}" 0.01, "10^{-1}" 0.1)
set yrange [0.00001:0.1]
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder

set title "UnionFind Decoder {/Symbol h} = 1e5"

set output "max_half_weight_eta1e5.eps"

plot "p_0.1.txt" using 1:6 with linespoints lt rgb "red" linewidth 5 pointtype 6 pointsize 1.5 title "p = 0.1",\
    "p_0.05.txt" using 1:6 with linespoints lt rgb "blue" linewidth 5 pointtype 2 pointsize 1.5 title "p = 0.05",\
    "p_0.02.txt" using 1:6 with linespoints lt rgb "green" linewidth 5 pointtype 6 pointsize 1.5 title "p = 0.02"

system("ps2pdf -dEPSCrop max_half_weight_eta1e5.eps max_half_weight_eta1e5.pdf")

set size 1,0.75
set output "max_half_weight_eta1e5_w.eps"
replot
system("ps2pdf -dEPSCrop max_half_weight_eta1e5_w.eps max_half_weight_eta1e5_w.pdf")

set size 1,0.6
set output "max_half_weight_eta1e5_w_w.eps"
replot
system("ps2pdf -dEPSCrop max_half_weight_eta1e5_w_w.eps max_half_weight_eta1e5_w_w.pdf")
