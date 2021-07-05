set terminal postscript eps color "Arial, 28"
set xlabel "maximum half weight" font "Arial, 28"
set ylabel "Logical Error Rate (p_L)" font "Arial, 28"
set grid ytics
set size 1,1

# data generating commands:

# RUST_BACKTRACE=full cargo run --release -- tool union_find_decoder_standard_xzzx_benchmark [7] [0.1,0.05,0.02] -p0 -b1000 -m100000000 -e100000 --bias_eta 10 --max_half_weight_eta10 1
# RUST_BACKTRACE=full cargo run --release -- tool union_find_decoder_standard_xzzx_benchmark [7] [0.1,0.05,0.02] -p0 -b1000 -m100000000 -e100000 --bias_eta 10 --max_half_weight_eta10 2
# RUST_BACKTRACE=full cargo run --release -- tool union_find_decoder_standard_xzzx_benchmark [7] [0.1,0.05,0.02] -p0 -b1000 -m100000000 -e100000 --bias_eta 10 --max_half_weight_eta10 3
# RUST_BACKTRACE=full cargo run --release -- tool union_find_decoder_standard_xzzx_benchmark [7] [0.1,0.05,0.02] -p0 -b1000 -m100000000 -e100000 --bias_eta 10 --max_half_weight_eta10 4
# RUST_BACKTRACE=full cargo run --release -- tool union_find_decoder_standard_xzzx_benchmark [7] [0.1,0.05,0.02] -p0 -b1000 -m100000000 -e100000 --bias_eta 10 --max_half_weight_eta10 5
# RUST_BACKTRACE=full cargo run --release -- tool union_find_decoder_standard_xzzx_benchmark [7] [0.1,0.05,0.02] -p0 -b1000 -m100000000 -e100000 --bias_eta 10 --max_half_weight_eta10 6
# RUST_BACKTRACE=full cargo run --release -- tool union_find_decoder_standard_xzzx_benchmark [7] [0.1,0.05,0.02] -p0 -b1000 -m100000000 -e100000 --bias_eta 10 --max_half_weight_eta10 7
# RUST_BACKTRACE=full cargo run --release -- tool union_find_decoder_standard_xzzx_benchmark [7] [0.1,0.05,0.02] -p0 -b1000 -m100000000 -e100000 --bias_eta 10 --max_half_weight_eta10 8
# RUST_BACKTRACE=full cargo run --release -- tool union_find_decoder_standard_xzzx_benchmark [7] [0.1,0.05,0.02] -p0 -b1000 -m100000000 -e100000 --bias_eta 10 --max_half_weight_eta10 9
# RUST_BACKTRACE=full cargo run --release -- tool union_find_decoder_standard_xzzx_benchmark [7] [0.1,0.05,0.02] -p0 -b1000 -m100000000 -e100000 --bias_eta 10 --max_half_weight_eta10 10

# python -c "for i in range(1, 11): print('\'%d\' %d' % tuple([i for j in range(2)]), end=', ')"
set xrange [1:10]
set xtics ('1' 1, '2' 2, '3' 3, '4' 4, '5' 5, '6' 6, '7' 7, '8' 8, '9' 9, '10' 10)
set logscale y
set ytics ("10^{-5}" 0.00001, "10^{-4}" 0.0001, "10^{-3}" 0.001, "10^{-2}" 0.01, "10^{-1}" 0.1)
set yrange [0.00001:0.1]
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder

set title "UnionFind Decoder {/Symbol h} = 10"

set output "max_half_weight_eta10.eps"

plot "p_0.1.txt" using 1:6 with linespoints lt rgb "red" linewidth 5 pointtype 6 pointsize 1.5 title "p = 0.1",\
    "p_0.05.txt" using 1:6 with linespoints lt rgb "blue" linewidth 5 pointtype 2 pointsize 1.5 title "p = 0.05",\
    "p_0.02.txt" using 1:6 with linespoints lt rgb "green" linewidth 5 pointtype 6 pointsize 1.5 title "p = 0.02"

set output '|ps2pdf -dEPSCrop max_half_weight_eta10.eps max_half_weight_eta10.pdf'
replot

set size 1,0.75
set output "max_half_weight_eta10_w.eps"
replot
set output '|ps2pdf -dEPSCrop max_half_weight_eta10_w.eps max_half_weight_eta10_w.pdf'
replot

set size 1,0.6
set output "max_half_weight_eta10_w_w.eps"
replot
set output '|ps2pdf -dEPSCrop max_half_weight_eta10_w_w.eps max_half_weight_eta10_w_w.pdf'
replot
