set terminal postscript eps color "Arial, 28"
set xlabel "Error Rate (p)" font "Arial, 28"
set ylabel "Logical Error Rate (p_L)" font "Arial, 28"
set grid ytics
set size 1,1

set logscale x
set xrange [0.0513:0.0527]
set xtics ("5.15%%" 0.0515, "5.2%%" 0.052, "5.25%%" 0.0525)
set logscale y
set ytics ("0.16" 0.16, "0.18" 0.18, "0.2" 0.2, "0.22" 0.22, "0.24" 0.24, "0.26" 0.26)
set yrange [0.16:0.26]
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder

set output "erasure_only_circuit_level_threshold.eps"

set title "Atomic Qubit Erasure Only Circuit-Level"

plot "threshold_11_11.txt" using 9:6 with linespoints lt rgb "red" linewidth 5 pointtype 6 pointsize 1.5 title "d = 11",\
    "threshold_15_15.txt" using 9:6 with linespoints lt rgb "blue" linewidth 5 pointtype 2 pointsize 1.5 title "d = 15",\
    "threshold_19_19.txt" using 9:6 with linespoints lt rgb "green" linewidth 5 pointtype 2 pointsize 1.5 title "d = 19"

system("ps2pdf -dEPSCrop erasure_only_circuit_level_threshold.eps erasure_only_circuit_level_threshold.pdf")

# set size 1,0.75
# set output "erasure_only_circuit_level_threshold_w.eps"
# replot
# system("ps2pdf -dEPSCrop erasure_only_circuit_level_threshold_w.eps erasure_only_circuit_level_threshold_w.pdf")

# set size 1,0.6
# set output "erasure_only_circuit_level_threshold_w_w.eps"
# replot
# system("ps2pdf -dEPSCrop erasure_only_circuit_level_threshold_w_w.eps erasure_only_circuit_level_threshold_w_w.pdf")
