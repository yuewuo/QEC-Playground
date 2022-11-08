set terminal postscript eps color "Arial, 28"
set xlabel "Error Rate (p / p_{th})" font "Arial, 28"
set ylabel "Logical Error Rate (p_L / p_{L.th})" font "Arial, 28"
set grid ytics
set size 1,1

set logscale x
set xrange [1e-4:0.3]
set xtics ("1e-4" 1e-4, "1e-3" 1e-3, "1e-2" 1e-2, "1e-1" 0.1)
set logscale y
set yrange [1e-7:1]
set ytics ("1e-6" 0.000001, "1e-4" 0.0001, "1e-2" 0.01, "1" 1)
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder

set output "absolute_error_rate.eps"

# set title "Atomic Qubit 95% Erasure + 5% Pauli Circuit-Level"


plot "pauli_ratio_0.txt" using 2:($8) with linespoints lt rgb "red" linewidth 3 pointtype 6 pointsize 1 title "R = 0",\
    "" using 2:($8):(($8)*(1-$10)):(($8)*(1+$10)) with errorbars lt rgb "red" linewidth 3 pointtype 6 pointsize 1 notitle,\
    "pauli_ratio_0.01.txt" using 2:($8) with linespoints lt rgb "blue" linewidth 3 pointtype 6 pointsize 1 title "R = 0.01",\
    "" using 2:($8):(($8)*(1-$10)):(($8)*(1+$10)) with errorbars lt rgb "blue" linewidth 3 pointtype 6 pointsize 1 notitle,\
    "pauli_ratio_0.02.txt" using 2:($8) with linespoints lt rgb "yellow" linewidth 3 pointtype 6 pointsize 1 title "R = 0.02",\
    "" using 2:($8):(($8)*(1-$10)):(($8)*(1+$10)) with errorbars lt rgb "yellow" linewidth 3 pointtype 6 pointsize 1 notitle,\
    "pauli_ratio_0.05.txt" using 2:($8) with linespoints lt rgb "green" linewidth 3 pointtype 6 pointsize 1 title "R = 0.05",\
    "" using 2:($8):(($8)*(1-$10)):(($8)*(1+$10)) with errorbars lt rgb "green" linewidth 3 pointtype 6 pointsize 1 notitle,\
    "pauli_ratio_0.1.txt" using 2:($8) with linespoints lt rgb "purple" linewidth 3 pointtype 6 pointsize 1 title "R = 0.1",\
    "" using 2:($8):(($8)*(1-$10)):(($8)*(1+$10)) with errorbars lt rgb "purple" linewidth 3 pointtype 6 pointsize 1 notitle,\
    "pauli_ratio_0.25.txt" using 2:($8) with linespoints lt rgb "red" linewidth 3 pointtype 6 pointsize 1 title "R = 0.25",\
    "" using 2:($8):(($8)*(1-$10)):(($8)*(1+$10)) with errorbars lt rgb "red" linewidth 3 pointtype 6 pointsize 1 notitle,\
    "pauli_ratio_0.5.txt" using 2:($8) with linespoints lt rgb "blue" linewidth 3 pointtype 6 pointsize 1 title "R = 0.5",\
    "" using 2:($8):(($8)*(1-$10)):(($8)*(1+$10)) with errorbars lt rgb "blue" linewidth 3 pointtype 6 pointsize 1 notitle,\
    "pauli_ratio_1.txt" using 2:($8) with linespoints lt rgb "yellow" linewidth 3 pointtype 6 pointsize 1 title "R = 1",\
    "" using 2:($8):(($8)*(1-$10)):(($8)*(1+$10)) with errorbars lt rgb "yellow" linewidth 3 pointtype 6 pointsize 1 notitle

system("ps2pdf -dEPSCrop absolute_error_rate.eps absolute_error_rate.pdf")

# set size 1,0.75
# set output "absolute_error_rate_w.eps"
# replot
# system("ps2pdf -dEPSCrop absolute_error_rate_w.eps absolute_error_rate_w.pdf")

# set size 1,0.6
# set output "absolute_error_rate_w_w.eps"
# replot
# system("ps2pdf -dEPSCrop absolute_error_rate_w_w.eps absolute_error_rate_w_w.pdf")
