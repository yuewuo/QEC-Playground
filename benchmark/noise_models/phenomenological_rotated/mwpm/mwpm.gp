set terminal postscript eps color "Arial, 28"
set xlabel "Error Rate (p)" font "Arial, 28"
set ylabel "Logical Error Rate (p_L)" font "Arial, 28"
set grid ytics
set size 1,1.2

set logscale x
set xrange [0.001:0.05]
set xtics ("0.001" 0.001, "0.002" 0.002, "0.005" 0.005, "0.01" 0.01, "0.02" 0.02, "0.05" 0.05)
set logscale y
set ytics ("10^{-8}" 0.00000001, "10^{-7}" 0.0000001, "10^{-6}" 0.000001, "10^{-5}" 0.00001, "10^{-4}" 0.0001, "10^{-3}" 0.001, "10^{-2}" 0.01, "10^{-1}" 0.1)
set yrange [0.00000001:1]
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder

set output "mwpm.eps"

set title "Phenomenological MWPM"

plot "d_3_3.txt" using 1:7 with linespoints lt rgb "red" linewidth 4 pointtype 2 pointsize 1 title "d = 3",\
    "" using 1:7:($7*(1-$9)):($7*(1+$9)) with errorbars lt rgb "red" linewidth 4 pointtype 2 pointsize 1 notitle,\
    "d_5_5.txt" using 1:7 with linespoints lt rgb "blue" linewidth 4 pointtype 2 pointsize 1 title "d = 5",\
    "" using 1:7:($7*(1-$9)):($7*(1+$9)) with errorbars lt rgb "blue" linewidth 4 pointtype 2 pointsize 1 notitle,\
    "d_7_7.txt" using 1:7 with linespoints lt rgb "green" linewidth 4 pointtype 2 pointsize 1 title "d = 7",\
    "" using 1:7:($7*(1-$9)):($7*(1+$9)) with errorbars lt rgb "green" linewidth 4 pointtype 2 pointsize 1 notitle,\
    "d_9_9.txt" using 1:7 with linespoints lt rgb "yellow" linewidth 4 pointtype 2 pointsize 1 title "d = 9",\
    "" using 1:7:($7*(1-$9)):($7*(1+$9)) with errorbars lt rgb "yellow" linewidth 4 pointtype 2 pointsize 1 notitle,\
    "d_11_11.txt" using 1:7 with linespoints lt rgb "purple" linewidth 4 pointtype 2 pointsize 1 title "d = 11",\
    "" using 1:7:($7*(1-$9)):($7*(1+$9)) with errorbars lt rgb "purple" linewidth 4 pointtype 2 pointsize 1 notitle,\
    "d_13_13.txt" using 1:7 with linespoints lt rgb "orange" linewidth 4 pointtype 2 pointsize 1 title "d = 13",\
    "" using 1:7:($7*(1-$9)):($7*(1+$9)) with errorbars lt rgb "orange" linewidth 4 pointtype 2 pointsize 1 notitle,

system("ps2pdf -dEPSCrop mwpm.eps mwpm.pdf")

# set size 1,0.75
# set output "mwpm_w.eps"
# replot
# system("ps2pdf -dEPSCrop mwpm_w.eps mwpm_w.pdf")

# set size 1,0.6
# set output "mwpm_w_w.eps"
# replot
# system("ps2pdf -dEPSCrop mwpm_w_w.eps mwpm_w_w.pdf")
