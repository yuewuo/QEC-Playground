set terminal postscript eps color "Arial, 28"
set xlabel "Code Distance (d)" font "Arial, 28"
set ylabel "Logical Error Rate (p_L)" font "Arial, 28"
set grid ytics
set size 1,1

set xrange [3:19]
set xtics ("3" 3, "5" 5, "7" 7, "9" 9, "11" 11, "13" 13, "15" 15, "17" 17, "19" 19)
set logscale y
set ytics ("10^{-8}" 0.00000001, "10^{-7}" 0.0000001, "10^{-6}" 0.000001, "10^{-5}" 0.00001, "10^{-4}" 0.0001, "10^{-3}" 0.001, "10^{-2}" 0.01, "10^{-1}" 0.1)
set yrange [0.00000001:1]
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder

set output "error_rate_0.002.eps"

set title "Atomic Qubit Compared with Pure Pauli (p = 0.002)"

plot "mixed_0.002.txt" using 3:7 with linespoints lt rgb "red" linewidth 5 pointtype 6 pointsize 1.5 title "95% erasure + 5% Pauli",\
    "pauli_0.002.txt" using 3:7 with linespoints lt rgb "blue" linewidth 5 pointtype 2 pointsize 1.5 title "pure Pauli"

set output '|ps2pdf -dEPSCrop error_rate_0.002.eps error_rate_0.002.pdf'
replot

# set size 1,0.75
# set output "error_rate_0.002_w.eps"
# replot
# set output '|ps2pdf -dEPSCrop error_rate_0.002_w.eps error_rate_0.002_w.pdf'
# replot

# set size 1,0.6
# set output "error_rate_0.002_w_w.eps"
# replot
# set output '|ps2pdf -dEPSCrop error_rate_0.002_w_w.eps error_rate_0.002_w_w.pdf'
# replot
