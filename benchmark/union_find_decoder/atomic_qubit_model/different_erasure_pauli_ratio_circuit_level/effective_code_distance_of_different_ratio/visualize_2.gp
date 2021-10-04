set terminal postscript eps color "Arial, 28"
set xlabel "Error Rate (p)" font "Arial, 28"
set ylabel "Logical Error Rate (p_L)" font "Arial, 28"
set grid ytics
set size 1,1

set logscale x
set xrange [0.0009:0.06]
set xtics ("0.001" 0.001, "0.003" 0.003, "0.01" 0.01, "0.03" 0.03)
set logscale y
set yrange [0.0005:0.5]
set ytics ("0.0005" 0.0005, "0.005" 0.005, "0.05" 0.05, "0.5" 0.5)
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder

set output "visualize_2.eps"

# set title "Atomic Qubit 95% Erasure + 5% Pauli Circuit-Level"

plot "pauli_ratio_0.txt" using 2:8 with linespoints lt rgb "red" linewidth 3 pointtype 6 pointsize 1 title "R = 0",\
    "" using 2:8:($8-$8*$10):($8+$8*$10) with errorbars lt rgb "red" linewidth 3 pointtype 6 pointsize 1 notitle,\
    "pauli_ratio_0.01.txt" using 2:8 with linespoints lt rgb "blue" linewidth 3 pointtype 6 pointsize 1 title "R = 0.01",\
    "" using 2:8:($8-$8*$10):($8+$8*$10) with errorbars lt rgb "blue" linewidth 3 pointtype 6 pointsize 1 notitle,\
    "pauli_ratio_0.05.txt" using 2:8 with linespoints lt rgb "green" linewidth 3 pointtype 6 pointsize 1 title "R = 0.05",\
    "" using 2:8:($8-$8*$10):($8+$8*$10) with errorbars lt rgb "green" linewidth 3 pointtype 6 pointsize 1 notitle,\
    "pauli_ratio_0.1.txt" using 2:8 with linespoints lt rgb "purple" linewidth 3 pointtype 6 pointsize 1 title "R = 0.1",\
    "" using 2:8:($8-$8*$10):($8+$8*$10) with errorbars lt rgb "purple" linewidth 3 pointtype 6 pointsize 1 notitle,\
    "pauli_ratio_1.txt" using 2:8 with linespoints lt rgb "yellow" linewidth 3 pointtype 6 pointsize 1 title "R = 1",\
    "" using 2:8:($8-$8*$10):($8+$8*$10) with errorbars lt rgb "yellow" linewidth 3 pointtype 6 pointsize 1 notitle

set output '|ps2pdf -dEPSCrop visualize_2.eps visualize_2.pdf'
replot

# set size 1,0.75
# set output "visualize_2_w.eps"
# replot
# set output '|ps2pdf -dEPSCrop visualize_2_w.eps visualize_2_w.pdf'
# replot

# set size 1,0.6
# set output "visualize_2_w_w.eps"
# replot
# set output '|ps2pdf -dEPSCrop visualize_2_w_w.eps visualize_2_w_w.pdf'
# replot
