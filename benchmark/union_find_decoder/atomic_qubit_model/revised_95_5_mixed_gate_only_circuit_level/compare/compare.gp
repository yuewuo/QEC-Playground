set terminal postscript eps color "Arial, 28"
set xlabel "Error Rate (p)" font "Arial, 28"
set ylabel "Logical Error Rate (p_L)" font "Arial, 28"
set grid ytics
set size 1,1

set logscale x
set xrange [0.00005:0.5]
set xtics ("5e-5" 0.00005, "5e-4" 0.0005, "5e-3" 0.005, "5e-2" 0.05, "0.5" 0.5)
set logscale y
set ytics ("10^{-8}" 0.00000001, "10^{-7}" 0.0000001, "10^{-6}" 0.000001, "10^{-5}" 0.00001, "10^{-4}" 0.0001, "10^{-3}" 0.001, "10^{-2}" 0.01, "10^{-1}" 0.1)
set yrange [0.00000001:1]
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder

set output "compare.eps"

set title "Atomic Qubit 95% Erasure + 5% Pauli Circuit-Level"

plot "mixed/d_3_3.txt" using 1:7 with linespoints lt rgb "red" linewidth 5 pointtype 2 pointsize 1.5 title "d = 3",\
    "mixed/d_5_5.txt" using 1:7 with linespoints lt rgb "blue" linewidth 5 pointtype 2 pointsize 1.5 title "d = 5",\
    "mixed/d_7_7.txt" using 1:7 with linespoints lt rgb "green" linewidth 5 pointtype 2 pointsize 1.5 title "d = 7",\
    "mixed/d_9_9.txt" using 1:7 with linespoints lt rgb "yellow" linewidth 5 pointtype 2 pointsize 1.5 title "d = 9",\
    "mixed/d_11_11.txt" using 1:7 with linespoints lt rgb "purple" linewidth 5 pointtype 2 pointsize 1.5 title "d = 11",\
    "mixed/d_13_13.txt" using 1:7 with linespoints lt rgb "orange" linewidth 5 pointtype 2 pointsize 1.5 title "d = 13",\
    "pauli_only/d_3_3.txt" using 1:7 with linespoints lt rgb "red" linewidth 5 pointtype 6 pointsize 1.5 notitle,\
    "pauli_only/d_5_5.txt" using 1:7 with linespoints lt rgb "blue" linewidth 5 pointtype 6 pointsize 1.5 notitle,\
    "pauli_only/d_7_7.txt" using 1:7 with linespoints lt rgb "green" linewidth 5 pointtype 6 pointsize 1.5 notitle,\
    "pauli_only/d_9_9.txt" using 1:7 with linespoints lt rgb "yellow" linewidth 5 pointtype 6 pointsize 1.5 notitle,\
    "pauli_only/d_11_11.txt" using 1:7 with linespoints lt rgb "purple" linewidth 5 pointtype 6 pointsize 1.5 notitle,\
    "pauli_only/d_13_13.txt" using 1:7 with linespoints lt rgb "orange" linewidth 5 pointtype 6 pointsize 1.5 notitle

system("ps2pdf -dEPSCrop compare.eps compare.pdf")

# set size 1,0.75
# set output "compare_w.eps"
# replot
# system("ps2pdf -dEPSCrop compare_w.eps compare_w.pdf")

# set size 1,0.6
# set output "compare_w_w.eps"
# replot
# system("ps2pdf -dEPSCrop compare_w_w.eps compare_w_w.pdf")
