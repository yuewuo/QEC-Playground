set terminal postscript eps color "Arial, 28"
set xlabel "Pauli Error Ratio" font "Arial, 28"
set ylabel "Threshold Error Rate (p_{th})" font "Arial, 28"
set grid ytics
set size 1,1

# set logscale x
set xrange [0:1]
set xtics ("0" 0, "0.2" 0.2, "0.4" 0.4, "0.6" 0.6, "0.8" 0.8, "1" 1)
# set logscale y
# set ytics ("0.005" 0.005, "0.01" 0.01, "0.02" 0.02, "0.04" 0.04)
# set yrange [0.005:0.06]
set yrange [0:0.055]
set ytics ("0.00" 0.00, "0.01" 0.01, "0.02" 0.02, "0.03" 0.03, "0.04" 0.04, "0.05" 0.05)
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder

set output "different_erasure_pauli_ratio_circuit_level.eps"

set title "Atomic Qubit Thresholds with Pauli Error Ratio"

plot "thresholds.txt" using 1:2 with linespoints lt rgb "red" linewidth 5 pointtype 6 pointsize 1.5 title "d = 3"

system("ps2pdf -dEPSCrop different_erasure_pauli_ratio_circuit_level.eps different_erasure_pauli_ratio_circuit_level.pdf")

# set size 1,0.75
# set output "different_erasure_pauli_ratio_circuit_level_w.eps"
# replot
# system("ps2pdf -dEPSCrop different_erasure_pauli_ratio_circuit_level_w.eps different_erasure_pauli_ratio_circuit_level_w.pdf")

# set size 1,0.6
# set output "different_erasure_pauli_ratio_circuit_level_w_w.eps"
# replot
# system("ps2pdf -dEPSCrop different_erasure_pauli_ratio_circuit_level_w_w.eps different_erasure_pauli_ratio_circuit_level_w_w.pdf")
