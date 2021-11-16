set terminal postscript eps color "Arial, 28"
set xlabel "Error Rate (p)" font "Arial, 28"
set ylabel "Logical Error Rate (p_L)" font "Arial, 28"
set grid ytics
set size 1,1

set logscale x
set xrange [0.0003:0.5]
set xtics ("5e-4" 0.0005, "5e-3" 0.005, "5e-2" 0.05, "0.5" 0.5)
set logscale y
set ytics ("1e-9" 0.000000001, "1e-8" 0.00000001, "1e-7" 0.0000001, "1e-6" 0.000001, "1e-5" 0.00001, "1e-4" 0.0001, "1e-3" 0.001, "1e-2" 0.01, "1e-1" 0.1)
set yrange [0.000000001:1]
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder

set output "find_turning_point.eps"

set title "Atomic Qubit Perfect Measurement (UF, d=3)"

plot "only_pauli.txt" using 1:7 with linespoints lt rgb "red" linewidth 5 pointtype 6 pointsize 1.5 title "only Pauli error",\
    "only_erasure.txt" using 1:7 with linespoints lt rgb "blue" linewidth 5 pointtype 2 pointsize 1.5 title "only erasure error",\
    "both.txt" using 1:7 with linespoints lt rgb "green" linewidth 5 pointtype 2 pointsize 1.5 title "both"

system("ps2pdf -dEPSCrop find_turning_point.eps find_turning_point.pdf")

# set size 1,0.75
# set output "find_turning_point_w.eps"
# replot
# system("ps2pdf -dEPSCrop find_turning_point_w.eps find_turning_point_w.pdf")

# set size 1,0.6
# set output "find_turning_point_w_w.eps"
# replot
# system("ps2pdf -dEPSCrop find_turning_point_w_w.eps find_turning_point_w_w.pdf")
