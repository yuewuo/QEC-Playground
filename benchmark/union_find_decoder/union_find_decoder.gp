set terminal postscript eps color "Arial, 28"
set xlabel "Depolarizing Error Rate (p)" font "Arial, 28"
set ylabel "Logical Error Rate (p_L)" font "Arial, 28"
set grid ytics
set size 1,1

# data generating commands:
# cargo run --release -- tool union_find_decoder_standard_planar_benchmark [3] [1e-1,5e-2,2e-2,1e-2,5e-3,2e-3,1e-3,5e-4,2e-4,1e-4,5e-5,2e-5,1e-5] -p0 -b1000 -m100000000 --only_count_logical_x
# cargo run --release -- tool union_find_decoder_standard_planar_benchmark [5] [1e-1,5e-2,2e-2,1e-2,5e-3,2e-3,1e-3,5e-4,2e-4] -b1000 -p0 -m100000000 --only_count_logical_x 
# cargo run --release -- tool union_find_decoder_standard_planar_benchmark [7] [1e-1,5e-2,2e-2,1e-2,5e-3,2e-3,1e-3] -p0 -m100000000 -b100 -e1000 --only_count_logical_x
# cargo run --release -- tool union_find_decoder_standard_planar_benchmark [9] [1e-1,5e-2,2e-2,1e-2,5e-3,2e-3] -p0 -m100000000 -b10 -e200 --only_count_logical_x
# cargo run --release -- tool union_find_decoder_standard_planar_benchmark [11] [1e-1,5e-2,2e-2,1e-2,5e-3,2e-3] -p0 -m100000000 -b10 -e200 --only_count_logical_x
# cargo run --release -- tool union_find_decoder_standard_planar_benchmark [13] [1e-1,5e-2,2e-2,1e-2,5e-3,2e-3] -p0 -m100000000 -b10 -e200 --only_count_logical_x

set logscale x
set xrange [0.00001:0.1]
set xtics ("10^{-5}" 0.00001, "10^{-4}" 0.0001, "10^{-3}" 0.001, "10^{-2}" 0.01, "10^{-1}" 0.1)
set logscale y
set ytics ("10^{-8}" 0.00000001, "10^{-7}" 0.0000001, "10^{-6}" 0.000001, "10^{-5}" 0.00001, "10^{-4}" 0.0001, "10^{-3}" 0.001, "10^{-2}" 0.01, "10^{-1}" 0.1)
set yrange [0.00000001:1]
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder

set output "union_find_decoder.eps"

plot "d_3.txt" using 1:5 with linespoints lt rgb "red" linewidth 5 pointtype 6 pointsize 1.5 title "d = 3",\
    "d_5.txt" using 1:5 with linespoints lt rgb "blue" linewidth 5 pointtype 2 pointsize 1.5 title "d = 5",\
    "d_7.txt" using 1:5 with linespoints lt rgb "green" linewidth 5 pointtype 2 pointsize 1.5 title "d = 7",\
    "d_9.txt" using 1:5 with linespoints lt rgb "yellow" linewidth 5 pointtype 2 pointsize 1.5 title "d = 9",\
    "d_11.txt" using 1:5 with linespoints lt rgb "purple" linewidth 5 pointtype 2 pointsize 1.5 title "d = 11",\
    "d_13.txt" using 1:5 with linespoints lt rgb "orange" linewidth 5 pointtype 2 pointsize 1.5 title "d = 13"

system("ps2pdf -dEPSCrop union_find_decoder.eps union_find_decoder.pdf")

set size 1,0.75
set output "union_find_decoder_w.eps"
replot
system("ps2pdf -dEPSCrop union_find_decoder_w.eps union_find_decoder_w.pdf")

set size 1,0.6
set output "union_find_decoder_w_w.eps"
replot
system("ps2pdf -dEPSCrop union_find_decoder_w_w.eps union_find_decoder_w_w.pdf")
