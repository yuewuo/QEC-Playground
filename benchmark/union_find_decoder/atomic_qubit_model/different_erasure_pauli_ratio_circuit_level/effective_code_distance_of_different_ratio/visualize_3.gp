set terminal postscript eps color "Arial, 28"
set xlabel "Error Rate (p / p_{th})" font "Arial, 28"
set ylabel "Logical Error Rate (p_L / p_{L.th})" font "Arial, 28"
set grid ytics
set size 1,1

set logscale x
set xrange [0.1:1]
set xtics ("0.1" 0.1, "0.2" 0.2, "0.3" 0.3, "0.4" 0.4, "0.6" 0.6, "0.8" 0.8, "1.0" 1.0)
set logscale y
set yrange [0.001:1]
set ytics ("0.001" 0.001, "0.01" 0.01, "0.1" 0.1, "1" 1)
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder

set output "visualize_3.eps"

# set title "Atomic Qubit 95% Erasure + 5% Pauli Circuit-Level"

pLth0 = "`head -1 pauli_ratio_0.txt | awk '{print $8}'`" + 0
pLth001 = "`head -1 pauli_ratio_0.01.txt | awk '{print $8}'`" + 0
pLth005 = "`head -1 pauli_ratio_0.05.txt | awk '{print $8}'`" + 0
pLth01 = "`head -1 pauli_ratio_0.1.txt | awk '{print $8}'`" + 0
pLth1 = "`head -1 pauli_ratio_1.txt | awk '{print $8}'`" + 0

plot "pauli_ratio_0.txt" using 1:($8/pLth0) with linespoints lt rgb "red" linewidth 3 pointtype 6 pointsize 1 title "R = 0",\
    "" using 1:($8/pLth0):(($8/pLth0)*(1-$10)):(($8/pLth0)*(1+$10)) with errorbars lt rgb "red" linewidth 3 pointtype 6 pointsize 1 notitle,\
    "pauli_ratio_0.01.txt" using 1:($8/pLth001) with linespoints lt rgb "blue" linewidth 3 pointtype 6 pointsize 1 title "R = 0.01",\
    "" using 1:($8/pLth001):(($8/pLth001)*(1-$10)):(($8/pLth001)*(1+$10)) with errorbars lt rgb "blue" linewidth 3 pointtype 6 pointsize 1 notitle,\
    "pauli_ratio_0.05.txt" using 1:($8/pLth005) with linespoints lt rgb "green" linewidth 3 pointtype 6 pointsize 1 title "R = 0.05",\
    "" using 1:($8/pLth005):(($8/pLth005)*(1-$10)):(($8/pLth005)*(1+$10)) with errorbars lt rgb "green" linewidth 3 pointtype 6 pointsize 1 notitle,\
    "pauli_ratio_0.1.txt" using 1:($8/pLth01) with linespoints lt rgb "purple" linewidth 3 pointtype 6 pointsize 1 title "R = 0.1",\
    "" using 1:($8/pLth01):(($8/pLth01)*(1-$10)):(($8/pLth01)*(1+$10)) with errorbars lt rgb "purple" linewidth 3 pointtype 6 pointsize 1 notitle,\
    "pauli_ratio_1.txt" using 1:($8/pLth1) with linespoints lt rgb "yellow" linewidth 3 pointtype 6 pointsize 1 title "R = 1",\
    "" using 1:($8/pLth1):(($8/pLth1)*(1-$10)):(($8/pLth1)*(1+$10)) with errorbars lt rgb "yellow" linewidth 3 pointtype 6 pointsize 1 notitle

set output '|ps2pdf -dEPSCrop visualize_3.eps visualize_3.pdf'
replot

# set size 1,0.75
# set output "visualize_3_w.eps"
# replot
# set output '|ps2pdf -dEPSCrop visualize_3_w.eps visualize_3_w.pdf'
# replot

# set size 1,0.6
# set output "visualize_3_w_w.eps"
# replot
# set output '|ps2pdf -dEPSCrop visualize_3_w_w.eps visualize_3_w_w.pdf'
# replot
