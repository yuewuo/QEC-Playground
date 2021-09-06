set terminal postscript eps color "Arial, 28"
set xlabel "Error Rate (p)" font "Arial, 28"
set ylabel "Logical Error Rate (p_L)" font "Arial, 28"
set grid ytics
set size 1,1

set logscale x
set xrange [0.0005:0.05]
set xtics ("5e-4" 0.0005, "5e-3" 0.005, "5e-2" 0.05)
set logscale y
set ytics ("10^{-5}" 0.00001, "10^{-4}" 0.0001, "10^{-3}" 0.001, "10^{-2}" 0.01, "10^{-1}" 0.1)
set yrange [0.00001:0.1]
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder

set output "fit_compare.eps"

set title "Atomic Qubit 95% Erasure + 5% Pauli Circuit-Level"

plot "fit_compare_d_3.txt" using 1:2 with linespoints lt rgb "red" linewidth 3 pointtype 6 pointsize 1 title "simulated d = 3",\
    "" using 1:2:($2-$2*$3):($2+$2*$3) with errorbars lt rgb "red" linewidth 3 pointtype 6 pointsize 1 notitle,\
    "" using 1:4 with linespoints lt rgb "light-red" linewidth 3 pointtype 2 pointsize 1 title "fitted d = 3",\
    "fit_compare_d_5.txt" using 1:2 with linespoints lt rgb "blue" linewidth 3 pointtype 6 pointsize 1 title "simulated d = 5",\
    "" using 1:2:($2-$2*$3):($2+$2*$3) with errorbars lt rgb "blue" linewidth 3 pointtype 6 pointsize 1 notitle,\
    "" using 1:4 with linespoints lt rgb "light-blue" linewidth 3 pointtype 2 pointsize 1 title "fitted d = 5",\
    "fit_compare_d_7.txt" using 1:2 with linespoints lt rgb "green" linewidth 3 pointtype 6 pointsize 1 title "simulated d = 7",\
    "" using 1:2:($2-$2*$3):($2+$2*$3) with errorbars lt rgb "green" linewidth 3 pointtype 6 pointsize 1 notitle,\
    "" using 1:4 with linespoints lt rgb "light-green" linewidth 3 pointtype 2 pointsize 1 title "fitted d = 7",\
    "fit_compare_d_9.txt" using 1:2 with linespoints lt rgb "orange" linewidth 3 pointtype 6 pointsize 1 title "simulated d = 9",\
    "" using 1:2:($2-$2*$3):($2+$2*$3) with errorbars lt rgb "orange" linewidth 3 pointtype 6 pointsize 1 notitle,\
    "" using 1:4 with linespoints lt rgb "yellow" linewidth 3 pointtype 2 pointsize 1 title "fitted d = 9"

set output '|ps2pdf -dEPSCrop fit_compare.eps fit_compare.pdf'
replot

# set size 1,0.75
# set output "fit_compare_w.eps"
# replot
# set output '|ps2pdf -dEPSCrop fit_compare_w.eps fit_compare_w.pdf'
# replot

# set size 1,0.6
# set output "fit_compare_w_w.eps"
# replot
# set output '|ps2pdf -dEPSCrop fit_compare_w_w.eps fit_compare_w_w.pdf'
# replot
