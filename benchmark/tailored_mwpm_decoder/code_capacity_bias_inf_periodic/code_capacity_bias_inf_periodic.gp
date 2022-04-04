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

set output "code_capacity_bias_inf_periodic.eps"

set title "Tailored Code Capacity"

plot "d_4.txt" using 1:6 with linespoints lt rgb "red" linewidth 4 pointtype 2 pointsize 1 notitle "d = 4",\
    "" using 1:6:($6*(1-$8)):($6*(1+$8)) with errorbars lt rgb "red" linewidth 4 pointtype 2 pointsize 1 notitle,\
    "d_6.txt" using 1:6 with linespoints lt rgb "blue" linewidth 4 pointtype 2 pointsize 1 notitle "d = 6",\
    "" using 1:6:($6*(1-$8)):($6*(1+$8)) with errorbars lt rgb "blue" linewidth 4 pointtype 2 pointsize 1 notitle,\
    "d_8.txt" using 1:6 with linespoints lt rgb "green" linewidth 4 pointtype 2 pointsize 1 notitle "d = 8",\
    "" using 1:6:($6*(1-$8)):($6*(1+$8)) with errorbars lt rgb "green" linewidth 4 pointtype 2 pointsize 1 notitle,\
    "d_10.txt" using 1:6 with linespoints lt rgb "yellow" linewidth 4 pointtype 2 pointsize 1 notitle "d = 10",\
    "" using 1:6:($6*(1-$8)):($6*(1+$8)) with errorbars lt rgb "yellow" linewidth 4 pointtype 2 pointsize 1 notitle,\
    "d_12.txt" using 1:6 with linespoints lt rgb "purple" linewidth 4 pointtype 2 pointsize 1 notitle "d = 12",\
    "" using 1:6:($6*(1-$8)):($6*(1+$8)) with errorbars lt rgb "purple" linewidth 4 pointtype 2 pointsize 1 notitle,\
    "d_14.txt" using 1:6 with linespoints lt rgb "black" linewidth 4 pointtype 2 pointsize 1 notitle "d = 14",\
    "" using 1:6:($6*(1-$8)):($6*(1+$8)) with errorbars lt rgb "black" linewidth 4 pointtype 2 pointsize 1 notitle

system("ps2pdf -dEPSCrop code_capacity_bias_inf_periodic.eps code_capacity_bias_inf_periodic.pdf")

# set size 1,0.75
# set output "code_capacity_bias_inf_periodic_w.eps"
# replot
# system("ps2pdf -dEPSCrop code_capacity_bias_inf_periodic_w.eps code_capacity_bias_inf_periodic_w.pdf")

# set size 1,0.6
# set output "code_capacity_bias_inf_periodic_w_w.eps"
# replot
# system("ps2pdf -dEPSCrop code_capacity_bias_inf_periodic_w_w.eps code_capacity_bias_inf_periodic_w_w.pdf")
