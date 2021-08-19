set terminal postscript eps color "Arial, 28"
set xlabel "Error Rate (p)" font "Arial, 28"
set ylabel "Logical Error Rate (p_L)" font "Arial, 28"
set grid ytics
set size 1,1

set logscale x
set xrange [0.23:0.27]
set xtics ("0.23" 0.23, "0.24" 0.24, "0.25" 0.25, "0.26" 0.26, "0.27" 0.27)
set logscale y
set ytics ("0.1" 0.1, "0.2" 0.2, "0.3" 0.3, "0.4" 0.4, "0.5" 0.5, "0.6" 0.6)
set yrange [0.1:0.6]
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder

set output "erasure_only_phenomenological_threshold.eps"

set title "Atomic Qubit Erasure Only Phenomenological"

plot "threshold_11_11.txt" using 1:6 with linespoints lt rgb "red" linewidth 5 pointtype 6 pointsize 1.5 title "d = 11",\
    "threshold_15_15.txt" using 1:6 with linespoints lt rgb "blue" linewidth 5 pointtype 2 pointsize 1.5 title "d = 15"

set output '|ps2pdf -dEPSCrop erasure_only_phenomenological_threshold.eps erasure_only_phenomenological_threshold.pdf'
replot

# set size 1,0.75
# set output "erasure_only_phenomenological_threshold_w.eps"
# replot
# set output '|ps2pdf -dEPSCrop erasure_only_phenomenological_threshold_w.eps erasure_only_phenomenological_threshold_w.pdf'
# replot

# set size 1,0.6
# set output "erasure_only_phenomenological_threshold_w_w.eps"
# replot
# set output '|ps2pdf -dEPSCrop erasure_only_phenomenological_threshold_w_w.eps erasure_only_phenomenological_threshold_w_w.pdf'
# replot
