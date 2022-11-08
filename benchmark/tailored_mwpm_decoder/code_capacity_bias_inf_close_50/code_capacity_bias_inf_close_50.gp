set terminal postscript eps color "Arial, 28"
set xlabel "Error Rate (p)" font "Arial, 28"
set ylabel "Logical Error Rate (p_L)" font "Arial, 28"
set grid ytics
set size 1,1

set logscale x
set xrange [0.4:0.5]
set xtics ("0.40" 0.40, "0.45" 0.45, "0.50" 0.50)
set logscale y
set ytics ("0.1" 0.1, "0.5" 0.5)
set yrange [0.08:1]
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder

set output "code_capacity_bias_inf_close_50.eps"

set title "Tailored Code Capacity"

plot "d_3.txt" using 1:6 with linespoints lt rgb "red" linewidth 4 pointtype 2 pointsize 1 notitle "d = 3",\
    "" using 1:6:($6*(1-$8)):($6*(1+$8)) with errorbars lt rgb "red" linewidth 4 pointtype 2 pointsize 1 notitle,\
    "d_5.txt" using 1:6 with linespoints lt rgb "blue" linewidth 4 pointtype 2 pointsize 1 notitle "d = 5",\
    "" using 1:6:($6*(1-$8)):($6*(1+$8)) with errorbars lt rgb "blue" linewidth 4 pointtype 2 pointsize 1 notitle,\
    "d_7.txt" using 1:6 with linespoints lt rgb "green" linewidth 4 pointtype 2 pointsize 1 notitle "d = 7",\
    "" using 1:6:($6*(1-$8)):($6*(1+$8)) with errorbars lt rgb "green" linewidth 4 pointtype 2 pointsize 1 notitle,\
    "d_9.txt" using 1:6 with linespoints lt rgb "yellow" linewidth 4 pointtype 2 pointsize 1 notitle "d = 9",\
    "" using 1:6:($6*(1-$8)):($6*(1+$8)) with errorbars lt rgb "yellow" linewidth 4 pointtype 2 pointsize 1 notitle,\
    "d_11.txt" using 1:6 with linespoints lt rgb "purple" linewidth 4 pointtype 2 pointsize 1 notitle "d = 11",\
    "" using 1:6:($6*(1-$8)):($6*(1+$8)) with errorbars lt rgb "purple" linewidth 4 pointtype 2 pointsize 1 notitle,\
    "d_13.txt" using 1:6 with linespoints lt rgb "black" linewidth 4 pointtype 2 pointsize 1 notitle "d = 13",\
    "" using 1:6:($6*(1-$8)):($6*(1+$8)) with errorbars lt rgb "black" linewidth 4 pointtype 2 pointsize 1 notitle

system("ps2pdf -dEPSCrop code_capacity_bias_inf_close_50.eps code_capacity_bias_inf_close_50.pdf")

# set size 1,0.75
# set output "code_capacity_bias_inf_close_50_w.eps"
# replot
# system("ps2pdf -dEPSCrop code_capacity_bias_inf_close_50_w.eps code_capacity_bias_inf_close_50_w.pdf")

# set size 1,0.6
# set output "code_capacity_bias_inf_close_50_w_w.eps"
# replot
# system("ps2pdf -dEPSCrop code_capacity_bias_inf_close_50_w_w.eps code_capacity_bias_inf_close_50_w_w.pdf")
