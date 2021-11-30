set terminal postscript eps color "Arial, 28"
set xlabel "Error Rate (p / p_{th})" font "Arial, 28"
set ylabel "Logical Error Rate (p_L / p_{L.th})" font "Arial, 28"
set grid ytics
set size 1,1

set logscale x
set xrange [0.015:1]
set xtics ("0.03" 0.03, "0.1" 0.1, "0.3" 0.3, "1.0" 1.0)
set logscale y
set yrange [0.000001:1]
set ytics ("1e-6" 0.000001, "1e-4" 0.0001, "1e-2" 0.01, "1" 1)
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder

set output "relative_error_rate.eps"

# set title "Atomic Qubit 95% Erasure + 5% Pauli Circuit-Level"

pLth0 = "`grep '^0 ' pth_L.txt | awk '{print $2}'`" + 0
pLth001 = "`grep '^0\.01 ' pth_L.txt | awk '{print $2}'`" + 0
pLth005 = "`grep '^0\.05 ' pth_L.txt | awk '{print $2}'`" + 0
pLth01 = "`grep '^0\.1 ' pth_L.txt | awk '{print $2}'`" + 0
pLth025 = "`grep '^0\.25 ' pth_L.txt | awk '{print $2}'`" + 0
pLth05 = "`grep '^0\.5 ' pth_L.txt | awk '{print $2}'`" + 0
pLth1 = "`grep '^1 ' pth_L.txt | awk '{print $2}'`" + 0

plot "pauli_ratio_0.txt" using 1:($8/pLth0) with linespoints lt rgb "red" linewidth 3 pointtype 6 pointsize 1 title "R = 0",\
    "" using 1:($8/pLth0):(($8/pLth0)*(1-$10)):(($8/pLth0)*(1+$10)) with errorbars lt rgb "red" linewidth 3 pointtype 6 pointsize 1 notitle,\
    "pauli_ratio_0.01.txt" using 1:($8/pLth001) with linespoints lt rgb "blue" linewidth 3 pointtype 6 pointsize 1 title "R = 0.01",\
    "" using 1:($8/pLth001):(($8/pLth001)*(1-$10)):(($8/pLth001)*(1+$10)) with errorbars lt rgb "blue" linewidth 3 pointtype 6 pointsize 1 notitle,\
    "pauli_ratio_0.05.txt" using 1:($8/pLth005) with linespoints lt rgb "green" linewidth 3 pointtype 6 pointsize 1 title "R = 0.05",\
    "" using 1:($8/pLth005):(($8/pLth005)*(1-$10)):(($8/pLth005)*(1+$10)) with errorbars lt rgb "green" linewidth 3 pointtype 6 pointsize 1 notitle,\
    "pauli_ratio_0.1.txt" using 1:($8/pLth01) with linespoints lt rgb "purple" linewidth 3 pointtype 6 pointsize 1 title "R = 0.1",\
    "" using 1:($8/pLth01):(($8/pLth01)*(1-$10)):(($8/pLth01)*(1+$10)) with errorbars lt rgb "purple" linewidth 3 pointtype 6 pointsize 1 notitle,\
    "pauli_ratio_0.25.txt" using 1:($8/pLth025) with linespoints lt rgb "red" linewidth 3 pointtype 6 pointsize 1 title "R = 0.25",\
    "" using 1:($8/pLth025):(($8/pLth025)*(1-$10)):(($8/pLth025)*(1+$10)) with errorbars lt rgb "red" linewidth 3 pointtype 6 pointsize 1 notitle,\
    "pauli_ratio_0.5.txt" using 1:($8/pLth05) with linespoints lt rgb "blue" linewidth 3 pointtype 6 pointsize 1 title "R = 0.5",\
    "" using 1:($8/pLth05):(($8/pLth05)*(1-$10)):(($8/pLth05)*(1+$10)) with errorbars lt rgb "blue" linewidth 3 pointtype 6 pointsize 1 notitle,\
    "pauli_ratio_1.txt" using 1:($8/pLth1) with linespoints lt rgb "yellow" linewidth 3 pointtype 6 pointsize 1 title "R = 1",\
    "" using 1:($8/pLth1):(($8/pLth1)*(1-$10)):(($8/pLth1)*(1+$10)) with errorbars lt rgb "yellow" linewidth 3 pointtype 6 pointsize 1 notitle

system("ps2pdf -dEPSCrop relative_error_rate.eps relative_error_rate.pdf")

# set size 1,0.75
# set output "relative_error_rate_w.eps"
# replot
# system("ps2pdf -dEPSCrop relative_error_rate_w.eps relative_error_rate_w.pdf")

# set size 1,0.6
# set output "relative_error_rate_w_w.eps"
# replot
# system("ps2pdf -dEPSCrop relative_error_rate_w_w.eps relative_error_rate_w_w.pdf")
