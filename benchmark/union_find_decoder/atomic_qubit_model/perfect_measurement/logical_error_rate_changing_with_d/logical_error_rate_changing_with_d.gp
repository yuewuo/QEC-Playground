set terminal postscript eps color "Arial, 28"
set xlabel "Error Rate (p)" font "Arial, 28"
set ylabel "Logical Error Rate (p_L)" font "Arial, 28"
set grid ytics
set size 1,1

set xrange [3:37]
set xtics ("5e-3" 0.005, "0.01" 0.01, "5e-2" 0.05, "0.1" 0.1, "0.5" 0.5)
set logscale y
set ytics ("10^{-5}" 0.00001, "10^{-4}" 0.0001, "10^{-3}" 0.001, "10^{-2}" 0.01, "10^{-1}" 0.1)
set yrange [0.00001:1]
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder

set output "logical_error_rate_changing_with_d.eps"

set title "Atomic Qubit Perfect Measurement (UnionFind Decoder)"

plot "p_0.4.txt" using 3:7 with linespoints lt rgb "red" linewidth 5 pointtype 6 pointsize 1.5 title "p = 0.4",\
    "p_0.3.txt" using 3:7 with linespoints lt rgb "blue" linewidth 5 pointtype 2 pointsize 1.5 title "p = 0.3",\
    "p_0.25.txt" using 3:7 with linespoints lt rgb "green" linewidth 5 pointtype 2 pointsize 1.5 title "p = 0.25"

system("ps2pdf -dEPSCrop logical_error_rate_changing_with_d.eps logical_error_rate_changing_with_d.pdf")

# set size 1,0.75
# set output "logical_error_rate_changing_with_d_w.eps"
# replot
# system("ps2pdf -dEPSCrop logical_error_rate_changing_with_d_w.eps logical_error_rate_changing_with_d_w.pdf")

# set size 1,0.6
# set output "logical_error_rate_changing_with_d_w_w.eps"
# replot
# system("ps2pdf -dEPSCrop logical_error_rate_changing_with_d_w_w.eps logical_error_rate_changing_with_d_w_w.pdf")
