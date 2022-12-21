set terminal postscript eps color "Arial, 28"
set xlabel "Error Rate (p)" font "Arial, 28"
set ylabel "Logical Error Rate (p_L)" font "Arial, 28"
set grid ytics
set size 1,1

set logscale x
set xrange [0.005:0.5]
set xtics ("5e-3" 0.005, "5e-2" 0.05, "0.5" 0.5)
set logscale y
set ytics ("10^{-8}" 0.00000001, "10^{-7}" 0.0000001, "10^{-6}" 0.000001, "10^{-5}" 0.00001, "10^{-4}" 0.0001, "10^{-3}" 0.001, "10^{-2}" 0.01, "10^{-1}" 0.1)
set yrange [0.00000001:1]
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder

set output "code_capacity_decoder_compare_bias_none.eps"

set title "Code Capacity (no bias) Tailored MWPM, HyperUF"

plot "tailored_mwpm/d_3.txt" using 1:6 with linespoints lt rgb "red" linewidth 4 pointtype 2 pointsize 1 notitle "d = 3",\
    "" using 1:6:($6*(1-$8)):($6*(1+$8)) with errorbars lt rgb "red" linewidth 4 pointtype 2 pointsize 1 notitle,\
    "tailored_mwpm/d_5.txt" using 1:6 with linespoints lt rgb "blue" linewidth 4 pointtype 2 pointsize 1 notitle "d = 5",\
    "" using 1:6:($6*(1-$8)):($6*(1+$8)) with errorbars lt rgb "blue" linewidth 4 pointtype 2 pointsize 1 notitle,\
    "tailored_mwpm/d_7.txt" using 1:6 with linespoints lt rgb "green" linewidth 4 pointtype 2 pointsize 1 notitle "d = 7",\
    "" using 1:6:($6*(1-$8)):($6*(1+$8)) with errorbars lt rgb "green" linewidth 4 pointtype 2 pointsize 1 notitle,\
    "tailored_mwpm/d_9.txt" using 1:6 with linespoints lt rgb "yellow" linewidth 4 pointtype 2 pointsize 1 notitle "d = 9",\
    "" using 1:6:($6*(1-$8)):($6*(1+$8)) with errorbars lt rgb "yellow" linewidth 4 pointtype 2 pointsize 1 notitle,\
    "tailored_mwpm/d_11.txt" using 1:6 with linespoints lt rgb "purple" linewidth 4 pointtype 2 pointsize 1 notitle "d = 11",\
    "" using 1:6:($6*(1-$8)):($6*(1+$8)) with errorbars lt rgb "purple" linewidth 4 pointtype 2 pointsize 1 notitle,\
    "tailored_mwpm/d_13.txt" using 1:6 with linespoints lt rgb "black" linewidth 4 pointtype 2 pointsize 1 notitle "d = 13",\
    "" using 1:6:($6*(1-$8)):($6*(1+$8)) with errorbars lt rgb "black" linewidth 4 pointtype 2 pointsize 1 notitle,\
    "hyper_union_find/d_3.txt" using 1:6 with linespoints lt rgb "red" dashtype 3 linewidth 4 pointtype 2 pointsize 1 notitle "d = 3",\
    "" using 1:6:($6*(1-$8)):($6*(1+$8)) with errorbars lt rgb "red" dashtype 3 linewidth 4 pointtype 2 pointsize 1 notitle,\
    "hyper_union_find/d_5.txt" using 1:6 with linespoints lt rgb "blue" dashtype 3 linewidth 4 pointtype 2 pointsize 1 notitle "d = 5",\
    "" using 1:6:($6*(1-$8)):($6*(1+$8)) with errorbars lt rgb "blue" dashtype 3 linewidth 4 pointtype 2 pointsize 1 notitle,\
    "hyper_union_find/d_7.txt" using 1:6 with linespoints lt rgb "green" dashtype 3 linewidth 4 pointtype 2 pointsize 1 notitle "d = 7",\
    "" using 1:6:($6*(1-$8)):($6*(1+$8)) with errorbars lt rgb "green" dashtype 3 linewidth 4 pointtype 2 pointsize 1 notitle,\
    "hyper_union_find/d_9.txt" using 1:6 with linespoints lt rgb "yellow" dashtype 3 linewidth 4 pointtype 2 pointsize 1 notitle "d = 9",\
    "" using 1:6:($6*(1-$8)):($6*(1+$8)) with errorbars lt rgb "yellow" dashtype 3 linewidth 4 pointtype 2 pointsize 1 notitle,\
    "hyper_union_find/d_11.txt" using 1:6 with linespoints lt rgb "purple" dashtype 3 linewidth 4 pointtype 2 pointsize 1 notitle "d = 11",\
    "" using 1:6:($6*(1-$8)):($6*(1+$8)) with errorbars lt rgb "purple" dashtype 3 linewidth 4 pointtype 2 pointsize 1 notitle,\
    "hyper_union_find/d_13.txt" using 1:6 with linespoints lt rgb "black" dashtype 3 linewidth 4 pointtype 2 pointsize 1 notitle "d = 13",\
    "" using 1:6:($6*(1-$8)):($6*(1+$8)) with errorbars lt rgb "black" dashtype 3 linewidth 4 pointtype 2 pointsize 1 notitle

system("ps2pdf -dEPSCrop code_capacity_decoder_compare_bias_none.eps code_capacity_decoder_compare_bias_none.pdf")

# set size 1,0.75
# set output "code_capacity_decoder_compare_bias_none_w.eps"
# replot
# system("ps2pdf -dEPSCrop code_capacity_decoder_compare_bias_none_w.eps code_capacity_decoder_compare_bias_none_w.pdf")

# set size 1,0.6
# set output "code_capacity_decoder_compare_bias_none_w_w.eps"
# replot
# system("ps2pdf -dEPSCrop code_capacity_decoder_compare_bias_none_w_w.eps code_capacity_decoder_compare_bias_none_w_w.pdf")
