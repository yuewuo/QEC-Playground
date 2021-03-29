set terminal postscript eps color "Arial, 28"
set xlabel "Code Distance (d)" font "Arial, 28"
set ylabel "Clock Cycles" font "Arial, 28"
set grid ytics
set size 1,1

# set logscale x
# set xrange [0.00001:0.1]
# set xtics ("10^{-5}" 0.00001, "10^{-4}" 0.0001, "10^{-3}" 0.001, "10^{-2}" 0.01, "10^{-1}" 0.1)
# set logscale y
# set ytics ("10^{-8}" 0.00000001, "10^{-7}" 0.0000001, "10^{-6}" 0.000001, "10^{-5}" 0.00001, "10^{-4}" 0.0001, "10^{-3}" 0.001, "10^{-2}" 0.01, "10^{-1}" 0.1)
# set yrange [0.00000001:1]
set key outside horizontal top center font "Arial, 18"

set style fill transparent solid 0.2 noborder

set output "clock_cycle.eps"

plot "results.txt" using 1:2 with linespoints lt rgb "red" linewidth 5 pointtype 6 pointsize 1.5 title "experiment threshold (w/o fast channel)",\
    "results.txt" using 1:3 with linespoints lt rgb "blue" linewidth 5 pointtype 2 pointsize 1.5 title "experiment worst (w/o fast channel)",\
    "results_fast_channel.txt" using 1:2 with linespoints lt rgb "purple" linewidth 5 pointtype 6 pointsize 1.5 title "experiment threshold (w/ fast channel)",\
    "results_fast_channel.txt" using 1:3 with linespoints lt rgb "black" linewidth 5 pointtype 2 pointsize 1.5 title "experiment worst (w/ fast channel)",\
    "results.txt" using 1:4 with linespoints lt rgb "green" linewidth 5 pointtype 2 pointsize 1.5 title "theoretical worst = d * (3d + 6)",\
    "budget.txt" using 1:2 with linespoints lt rgb "dark-yellow" linewidth 5 pointtype 3 pointsize 1.5 title "cycle budget (10MHz, 1.8us measurement)"

set output '|ps2pdf -dEPSCrop clock_cycle.eps clock_cycle.pdf'
replot

set size 1,0.75
set output "clock_cycle_w.eps"
replot
set output '|ps2pdf -dEPSCrop clock_cycle_w.eps clock_cycle_w.pdf'
replot

set size 1,0.6
set output "clock_cycle_w_w.eps"
replot
set output '|ps2pdf -dEPSCrop clock_cycle_w_w.eps clock_cycle_w_w.pdf'
replot
