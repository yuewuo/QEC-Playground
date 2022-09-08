set terminal postscript eps color "Arial, 28"
set xlabel "Bias: {/Symbol h}" font "Arial, 28"
set ylabel "Threshold error rate: p_{th}" font "Arial, 28"
set grid ytics
set size 1,1

set logscale x
set xrange [0.5:1000]
set xtics ("0.5" 0.5, "1" 1, "3" 3, "10" 10, "30" 30, "100" 100, "300" 300)
# set logscale y
set ytics ("0.03" 0.03, "0.04" 0.04, "0.05" 0.05, "0.06" 0.06)
set yrange [0.03:0.07]
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder

set output "phenomenological_thresholds.eps"

# set title "Tailored Phenomenological"

plot "thresholds.txt" using 1:2 with linespoints lt rgb "red" linewidth 4 pointtype 2 pointsize 1 notitle "d = 3",\
    "" using 1:2:($2-$3):($2+$3) with errorbars lt rgb "red" linewidth 4 pointtype 2 pointsize 1 notitle

system("ps2pdf -dEPSCrop phenomenological_thresholds.eps phenomenological_thresholds.pdf")

# set size 1,0.75
# set output "phenomenological_thresholds_w.eps"
# replot
# system("ps2pdf -dEPSCrop phenomenological_thresholds_w.eps phenomenological_thresholds_w.pdf")

# set size 1,0.6
# set output "phenomenological_thresholds_w_w.eps"
# replot
# system("ps2pdf -dEPSCrop phenomenological_thresholds_w_w.eps phenomenological_thresholds_w_w.pdf")
