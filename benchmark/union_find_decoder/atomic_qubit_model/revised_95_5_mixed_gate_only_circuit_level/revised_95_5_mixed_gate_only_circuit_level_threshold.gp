set terminal postscript eps color "Arial, 28"
set xlabel "Error Rate (p)" font "Arial, 28"
set ylabel "Logical Error Rate (p_L)" font "Arial, 28"
set grid ytics
set size 1,1

set logscale x
set xrange [0.0307:0.0316]
set xtics ("3.1%%" 0.031, "3.15%%" 0.0315)
set logscale y
set ytics ("0.07" 0.07, "0.08" 0.08, "0.09" 0.09, "0.1" 0.1)
set yrange [0.07:0.1]
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder

set output "revised_95_5_mixed_gate_only_circuit_level_threshold.eps"

set title "Atomic Qubit 95% Erasure + 5% Pauli Circuit-Level"

plot "threshold_11_11.txt" using 9:6 with linespoints lt rgb "red" linewidth 5 pointtype 6 pointsize 1.5 title "d = 11",\
    "threshold_15_15.txt" using 9:6 with linespoints lt rgb "blue" linewidth 5 pointtype 2 pointsize 1.5 title "d = 15",\
    "threshold_19_19.txt" using 9:6 with linespoints lt rgb "green" linewidth 5 pointtype 2 pointsize 1.5 title "d = 19"

set output '|ps2pdf -dEPSCrop revised_95_5_mixed_gate_only_circuit_level_threshold.eps revised_95_5_mixed_gate_only_circuit_level_threshold.pdf'
replot

# set size 1,0.75
# set output "revised_95_5_mixed_gate_only_circuit_level_threshold_w.eps"
# replot
# set output '|ps2pdf -dEPSCrop revised_95_5_mixed_gate_only_circuit_level_threshold_w.eps revised_95_5_mixed_gate_only_circuit_level_threshold_w.pdf'
# replot

# set size 1,0.6
# set output "revised_95_5_mixed_gate_only_circuit_level_threshold_w_w.eps"
# replot
# set output '|ps2pdf -dEPSCrop revised_95_5_mixed_gate_only_circuit_level_threshold_w_w.eps revised_95_5_mixed_gate_only_circuit_level_threshold_w_w.pdf'
# replot
