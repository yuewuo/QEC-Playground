set terminal postscript eps color "Arial, 28"
set xlabel "Depolarizing Error Rate (p)" font "Arial, 28"
set ylabel "Logical Error Rate (p_L)" font "Arial, 28"
set grid ytics
set size 1,1

# data generating commands:
# cargo run --release -- tool distributed_union_find_decoder_standard_planar_benchmark [3,5,7,9,11,13] [1e-2] -p0 -b100 -m100000000 -e100000000 --only_count_logical_x --output_cycle_distribution
# cargo run --release -- tool distributed_union_find_decoder_standard_planar_benchmark [15,17,19] [1e-2] -p0 -b100 -m10000000000 -e100 --only_count_logical_x --output_cycle_distribution

set logscale x
set xrange [0.00001:0.1]
set xtics ("10^{-5}" 0.00001, "10^{-4}" 0.0001, "10^{-3}" 0.001, "10^{-2}" 0.01, "10^{-1}" 0.1)
set logscale y
set ytics ("10^{-8}" 0.00000001, "10^{-7}" 0.0000001, "10^{-6}" 0.000001, "10^{-5}" 0.00001, "10^{-4}" 0.0001, "10^{-3}" 0.001, "10^{-2}" 0.01, "10^{-1}" 0.1)
set yrange [0.00000001:1]
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder

set output "distributed_uf_decoder.eps"

plot "d_3.txt" using 1:5 with linespoints lt rgb "red" linewidth 5 pointtype 6 pointsize 1.5 title "d = 3",\
    "d_5.txt" using 1:5 with linespoints lt rgb "blue" linewidth 5 pointtype 2 pointsize 1.5 title "d = 5",\
    "d_7.txt" using 1:5 with linespoints lt rgb "green" linewidth 5 pointtype 2 pointsize 1.5 title "d = 7",\
    "d_9.txt" using 1:5 with linespoints lt rgb "yellow" linewidth 5 pointtype 2 pointsize 1.5 title "d = 9",\
    "d_11.txt" using 1:5 with linespoints lt rgb "purple" linewidth 5 pointtype 2 pointsize 1.5 title "d = 11",\
    "d_13.txt" using 1:5 with linespoints lt rgb "orange" linewidth 5 pointtype 2 pointsize 1.5 title "d = 13"

set output '|ps2pdf -dEPSCrop distributed_uf_decoder.eps distributed_uf_decoder.pdf'
replot

set size 1,0.75
set output "distributed_uf_decoder_w.eps"
replot
set output '|ps2pdf -dEPSCrop distributed_uf_decoder_w.eps distributed_uf_decoder_w.pdf'
replot

set size 1,0.6
set output "distributed_uf_decoder_w_w.eps"
replot
set output '|ps2pdf -dEPSCrop distributed_uf_decoder_w_w.eps distributed_uf_decoder_w_w.pdf'
replot
