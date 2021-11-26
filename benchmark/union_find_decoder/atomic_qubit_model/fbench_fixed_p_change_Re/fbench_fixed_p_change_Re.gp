set terminal postscript eps color "Arial, 28"
set xlabel "Erasure Ratio (R_e)" font "Arial, 28"
set ylabel "Logical Error Rate (p_L)" font "Arial, 28"
set grid ytics
set size 1,1

set xrange [0:1]
set xtics ("0" 0, "0.2" 0.2, "0.4" 0.4, "0.6" 0.6, "0.8" 0.8, "1" 1)
set logscale y
# print(", ".join([f'"10^{{-{e}}}" 1e-{e}' for e in range(10, 53, 10)]))
set ytics ("10^{-10}" 1e-10, "10^{-20}" 1e-20, "10^{-30}" 1e-30, "10^{-40}" 1e-40, "10^{-50}" 1e-50)
set yrange [1e-55:1e-5]
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder

set output "fbench_fixed_p_change_Re.eps"

set title "FBench Fixed p Change Re (UF Decoder)"

plot "d_3_3.txt" using 1:9 with linespoints lt rgb "red" linewidth 4 pointtype 2 pointsize 1 title "d = 3",\
    "" using 1:9:($9*(1-$10)):($9*(1+$10)) with errorbars lt rgb "red" linewidth 4 pointtype 2 pointsize 1 notitle,\
    "d_5_5.txt" using 1:9 with linespoints lt rgb "blue" linewidth 4 pointtype 2 pointsize 1 title "d = 5",\
    "" using 1:9:($9*(1-$10)):($9*(1+$10)) with errorbars lt rgb "blue" linewidth 4 pointtype 2 pointsize 1 notitle,\
    "d_7_7.txt" using 1:9 with linespoints lt rgb "green" linewidth 4 pointtype 2 pointsize 1 title "d = 7",\
    "" using 1:9:($9*(1-$10)):($9*(1+$10)) with errorbars lt rgb "green" linewidth 4 pointtype 2 pointsize 1 notitle,\
    "d_9_9.txt" using 1:9 with linespoints lt rgb "yellow" linewidth 4 pointtype 2 pointsize 1 title "d = 9",\
    "" using 1:9:($9*(1-$10)):($9*(1+$10)) with errorbars lt rgb "yellow" linewidth 4 pointtype 2 pointsize 1 notitle,\
    "d_11_11.txt" using 1:9 with linespoints lt rgb "purple" linewidth 4 pointtype 2 pointsize 1 title "d = 11",\
    "" using 1:9:($9*(1-$10)):($9*(1+$10)) with errorbars lt rgb "purple" linewidth 4 pointtype 2 pointsize 1 notitle,\
    "d_13_13.txt" using 1:9 with linespoints lt rgb "orange" linewidth 4 pointtype 2 pointsize 1 title "d = 13",\
    "" using 1:9:($9*(1-$10)):($9*(1+$10)) with errorbars lt rgb "orange" linewidth 4 pointtype 2 pointsize 1 notitle,

system("ps2pdf -dEPSCrop fbench_fixed_p_change_Re.eps fbench_fixed_p_change_Re.pdf")

# set size 1,0.75
# set output "fbench_fixed_p_change_Re_w.eps"
# replot
# system("ps2pdf -dEPSCrop fbench_fixed_p_change_Re_w.eps fbench_fixed_p_change_Re_w.pdf")

# set size 1,0.6
# set output "fbench_fixed_p_change_Re_w_w.eps"
# replot
# system("ps2pdf -dEPSCrop fbench_fixed_p_change_Re_w_w.eps fbench_fixed_p_change_Re_w_w.pdf")
