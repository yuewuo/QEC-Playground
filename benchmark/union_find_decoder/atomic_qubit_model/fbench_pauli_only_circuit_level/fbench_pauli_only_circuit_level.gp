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

set output "fbench_pauli_only_circuit_level.eps"

set title "FBench Pauli Only Circuit-Level"

plot "d_3_3.txt" using 1:9 with linespoints lt rgb "red" linewidth 4 pointtype 2 pointsize 1 title "d = 3",\
    "" using 1:9:($9*(1-$10)):($9*(1+$10)) with errorbars lt rgb "red" linewidth 4 pointtype 2 pointsize 1 notitle,\
    "d_5_5.txt" using 1:9 with linespoints lt rgb "blue" linewidth 4 pointtype 2 pointsize 1 title "d = 5",\
    "" using 1:9:($9*(1-$10)):($9*(1+$10)) with errorbars lt rgb "blue" linewidth 4 pointtype 2 pointsize 1 notitle,\
    "d_7_7.txt" using 1:9 with linespoints lt rgb "green" linewidth 4 pointtype 2 pointsize 1 title "d = 7",\
    "" using 1:9:($9*(1-$10)):($9*(1+$10)) with errorbars lt rgb "green" linewidth 4 pointtype 2 pointsize 1 notitle,\
    "d_9_9.txt" using 1:9 with linespoints lt rgb "yellow" linewidth 4 pointtype 2 pointsize 1 title "d = 9",\
    "" using 1:9:($9*(1-$10)):($9*(1+$10)) with errorbars lt rgb "yellow" linewidth 4 pointtype 2 pointsize 1 notitle,\
    "d_11_11.txt" using 1:9 with linespoints lt rgb "purple" linewidth 4 pointtype 2 pointsize 1 title "d = 11",\
    "" using 1:9:($9*(1-$10)):($9*(1+$10)) with errorbars lt rgb "purple" linewidth 4 pointtype 2 pointsize 1 notitle,\
    "d_13_13.txt" using 1:9 with linespoints lt rgb "orange" linewidth 4 pointtype 2 pointsize 1 title "d = 13",\
    "" using 1:9:($9*(1-$10)):($9*(1+$10)) with errorbars lt rgb "orange" linewidth 4 pointtype 2 pointsize 1 notitle,

system("ps2pdf -dEPSCrop fbench_pauli_only_circuit_level.eps fbench_pauli_only_circuit_level.pdf")

# set size 1,0.75
# set output "fbench_pauli_only_circuit_level_w.eps"
# replot
# system("ps2pdf -dEPSCrop fbench_pauli_only_circuit_level_w.eps fbench_pauli_only_circuit_level_w.pdf")

# set size 1,0.6
# set output "fbench_pauli_only_circuit_level_w_w.eps"
# replot
# system("ps2pdf -dEPSCrop fbench_pauli_only_circuit_level_w_w.eps fbench_pauli_only_circuit_level_w_w.pdf")
