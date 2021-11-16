set terminal postscript eps color "Arial, 28"
set xlabel "Error Rate (p / p_{th})" font "Arial, 28"
set ylabel "Logical Error Rate (p_L)" font "Arial, 28"
set grid ytics
set size 1,1

set logscale x
set xrange [0.1:1]
set xtics ("0.1" 0.1, "0.2" 0.2, "0.3" 0.3, "0.4" 0.4, "0.6" 0.6, "0.8" 0.8, "1.0" 1.0)
set logscale y
set yrange [0.0005:0.5]
set ytics ("0.0005" 0.0005, "0.005" 0.005, "0.05" 0.05, "0.5" 0.5)
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder

set output "visualize.eps"

# set title "Atomic Qubit 95% Erasure + 5% Pauli Circuit-Level"

plot "pauli_ratio_0.txt" using 1:8 with linespoints lt rgb "red" linewidth 3 pointtype 6 pointsize 1 title "R = 0",\
    "" using 1:8:($8-$8*$10):($8+$8*$10) with errorbars lt rgb "red" linewidth 3 pointtype 6 pointsize 1 notitle,\
    "pauli_ratio_0.01.txt" using 1:8 with linespoints lt rgb "blue" linewidth 3 pointtype 6 pointsize 1 title "R = 0.01",\
    "" using 1:8:($8-$8*$10):($8+$8*$10) with errorbars lt rgb "blue" linewidth 3 pointtype 6 pointsize 1 notitle,\
    "pauli_ratio_0.05.txt" using 1:8 with linespoints lt rgb "green" linewidth 3 pointtype 6 pointsize 1 title "R = 0.05",\
    "" using 1:8:($8-$8*$10):($8+$8*$10) with errorbars lt rgb "green" linewidth 3 pointtype 6 pointsize 1 notitle,\
    "pauli_ratio_0.1.txt" using 1:8 with linespoints lt rgb "purple" linewidth 3 pointtype 6 pointsize 1 title "R = 0.1",\
    "" using 1:8:($8-$8*$10):($8+$8*$10) with errorbars lt rgb "purple" linewidth 3 pointtype 6 pointsize 1 notitle,\
    "pauli_ratio_1.txt" using 1:8 with linespoints lt rgb "yellow" linewidth 3 pointtype 6 pointsize 1 title "R = 1",\
    "" using 1:8:($8-$8*$10):($8+$8*$10) with errorbars lt rgb "yellow" linewidth 3 pointtype 6 pointsize 1 notitle

system("ps2pdf -dEPSCrop visualize.eps visualize.pdf")

# set size 1,0.75
# set output "visualize_w.eps"
# replot
# system("ps2pdf -dEPSCrop visualize_w.eps visualize_w.pdf")

# set size 1,0.6
# set output "visualize_w_w.eps"
# replot
# system("ps2pdf -dEPSCrop visualize_w_w.eps visualize_w_w.pdf")
