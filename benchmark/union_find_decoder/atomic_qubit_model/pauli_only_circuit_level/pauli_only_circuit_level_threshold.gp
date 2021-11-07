set terminal postscript eps color "Arial, 28"
set xlabel "Error Rate (p)" font "Arial, 28"
set ylabel "Logical Error Rate (p_L)" font "Arial, 28"
set grid ytics
set size 1,1

set logscale x
set xrange [0.00568:0.00583]
set xtics ("0.57%%" 0.0057, "0.575%%" 0.00575, "0.58%%" 0.0058)
set logscale y
set ytics ("0.11" 0.11, "0.115" 0.115, "0.12" 0.12)
set yrange [0.107:0.125]
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder

set output "pauli_only_circuit_level_threshold.eps"

set title "Atomic Qubit Pauli Only Circuit-Level"

plot "threshold_11_11.txt" using 9:6 with linespoints lt rgb "red" linewidth 5 pointtype 6 pointsize 1.5 title "d = 11",\
    "threshold_15_15.txt" using 9:6 with linespoints lt rgb "blue" linewidth 5 pointtype 2 pointsize 1.5 title "d = 15",\
    "threshold_19_19.txt" using 9:6 with linespoints lt rgb "green" linewidth 5 pointtype 2 pointsize 1.5 title "d = 19"

system("ps2pdf -dEPSCrop pauli_only_circuit_level_threshold.eps pauli_only_circuit_level_threshold.pdf")

# set size 1,0.75
# set output "pauli_only_circuit_level_threshold_w.eps"
# replot
# system("ps2pdf -dEPSCrop pauli_only_circuit_level_threshold_w.eps pauli_only_circuit_level_threshold_w.pdf")

# set size 1,0.6
# set output "pauli_only_circuit_level_threshold_w_w.eps"
# replot
# system("ps2pdf -dEPSCrop pauli_only_circuit_level_threshold_w_w.eps pauli_only_circuit_level_threshold_w_w.pdf")
