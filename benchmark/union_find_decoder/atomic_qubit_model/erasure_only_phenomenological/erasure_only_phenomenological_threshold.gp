set terminal postscript eps color "Arial, 28"
set xlabel "Error Rate (p)" font "Arial, 28"
set ylabel "Logical Error Rate (p_L)" font "Arial, 28"
set grid ytics
set size 1,1

set logscale x
set xrange [0.245:0.251]
set xtics ("0.245" 0.245, "0.247" 0.247, "0.249" 0.249, "0.251" 0.251)
set logscale y
set ytics ("0.2" 0.2, "0.24" 0.24, "0.27" 0.27, "0.3" 0.3)
set yrange [0.2:0.3]
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder

set output "erasure_only_phenomenological_threshold.eps"

set title "Atomic Qubit Erasure Only Phenomenological"

plot "threshold_11_11.txt" using 9:6 with linespoints lt rgb "red" linewidth 5 pointtype 6 pointsize 1.5 title "d = 11",\
    "threshold_15_15.txt" using 9:6 with linespoints lt rgb "blue" linewidth 5 pointtype 2 pointsize 1.5 title "d = 15",\
    "threshold_19_19.txt" using 9:6 with linespoints lt rgb "green" linewidth 5 pointtype 2 pointsize 1.5 title "d = 19"

system("ps2pdf -dEPSCrop erasure_only_phenomenological_threshold.eps erasure_only_phenomenological_threshold.pdf")

# set size 1,0.75
# set output "erasure_only_phenomenological_threshold_w.eps"
# replot
# system("ps2pdf -dEPSCrop erasure_only_phenomenological_threshold_w.eps erasure_only_phenomenological_threshold_w.pdf")

# set size 1,0.6
# set output "erasure_only_phenomenological_threshold_w_w.eps"
# replot
# system("ps2pdf -dEPSCrop erasure_only_phenomenological_threshold_w_w.eps erasure_only_phenomenological_threshold_w_w.pdf")
