set terminal postscript eps color "Arial, 28"
set xlabel "Depolarizing probability (p)" font "Arial, 28"
set ylabel "Logical X error rate (p_L)" font "Arial, 28"
set grid ytics
set size 1,1

# data generating commands:
# cargo run --release -- tool automatic_benchmark [3] [5e-1,2e-1,1e-1,5e-2,2e-2,1e-2,5e-3,2e-3,1e-3,5e-4,2e-4,1e-4,5e-5] -q stupid_decoder
# cargo run --release -- tool automatic_benchmark [5] [5e-1,2e-1,1e-1,5e-2,2e-2,1e-2,5e-3,2e-3,1e-3,5e-4,2e-4] -q stupid_decoder
# cargo run --release -- tool automatic_benchmark [7] [5e-1,2e-1,1e-1,5e-2,2e-2,1e-2,5e-3,2e-3,1e-3,5e-4] -q stupid_decoder
# cargo run --release -- tool automatic_benchmark [9] [5e-1,2e-1,1e-1,5e-2,2e-2,1e-2,5e-3,2e-3,1e-3,5e-4] -q stupid_decoder
# cargo run --release -- tool automatic_benchmark [11] [5e-1,2e-1,1e-1,5e-2,2e-2,1e-2,5e-3,2e-3,1e-3,5e-4] -q stupid_decoder
# cargo run --release -- tool automatic_benchmark [13] [5e-1,2e-1,1e-1,5e-2,2e-2,1e-2,5e-3,2e-3] -q stupid_decoder
# cargo run --release -- tool automatic_benchmark [15] [5e-1,2e-1,1e-1,5e-2,2e-2,1e-2,5e-3,2e-3] -q stupid_decoder
# cargo run --release -- tool automatic_benchmark [25] [5e-1,2e-1,1e-1,5e-2,2e-2,1e-2,5e-3,2e-3] -q stupid_decoder -m 1000000

set logscale x
set xrange [0.00001:0.5]
set xtics ("10^{-5}" 0.00001, "10^{-4}" 0.0001, "10^{-3}" 0.001, "10^{-2}" 0.01, "10^{-1}" 0.1)
set logscale y
set ytics ("10^{-8}" 0.00000001, "10^{-7}" 0.0000001, "10^{-6}" 0.000001, "10^{-5}" 0.00001, "10^{-4}" 0.0001, "10^{-3}" 0.001, "10^{-2}" 0.01, "10^{-1}" 0.1)
set yrange [0.00000001:1]
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder

set output "stupid_decoder.eps"

plot "d_3.txt" using 1:5 with linespoints lt rgb "red" linewidth 5 pointtype 6 pointsize 1.5 title "d = 3",\
    "d_5.txt" using 1:5 with linespoints lt rgb "blue" linewidth 5 pointtype 2 pointsize 1.5 title "d = 5",\
    "d_7.txt" using 1:5 with linespoints lt rgb "green" linewidth 5 pointtype 2 pointsize 1.5 title "d = 7",\
    "d_9.txt" using 1:5 with linespoints lt rgb "yellow" linewidth 5 pointtype 2 pointsize 1.5 title "d = 9",\
    "d_11.txt" using 1:5 with linespoints lt rgb "purple" linewidth 5 pointtype 2 pointsize 1.5 title "d = 11",\
    "d_25.txt" using 1:5 with linespoints lt rgb "black" linewidth 5 pointtype 2 pointsize 1.5 title "d = 25"
    # "d_15.txt" using 1:5 with linespoints lt rgb "orange" linewidth 5 pointtype 2 pointsize 1.5 title "d = 15",\

set output '|ps2pdf -dEPSCrop stupid_decoder.eps stupid_decoder.pdf'
replot

set size 1,0.75
set output "stupid_decoder_w.eps"
replot
set output '|ps2pdf -dEPSCrop stupid_decoder_w.eps stupid_decoder_w.pdf'
replot

set size 1,0.6
set output "stupid_decoder_w_w.eps"
replot
set output '|ps2pdf -dEPSCrop stupid_decoder_w_w.eps stupid_decoder_w_w.pdf'
replot
