set terminal postscript eps color "Arial, 28"
set xlabel "Error Rate (p)" font "Arial, 28"
set ylabel "Logical Error Rate (p_L)" font "Arial, 28"
set grid ytics
set size 1,1

set logscale x
set xrange [5e-32:0.5]
set xtics ("1e-30" 5e-30, "1e-20" 1e-20, "1e-10" 1e-10, "0.5" 0.5)
set logscale y
set ytics ("1e-200" 1e-200, "1e-150" 1e-150, "1e-100" 1e-100, "1e-50" 1e-50, "1" 1)
set yrange [1e-230:1]
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder

set output "init_measurement_error_0.01_fbench_fake.eps"

set title "FBench Fake Init Measure Error 0.01 (0.98p erasure + 0.02p Pauli)" font "Arial, 18"

plot "d_3.txt" using 1:9 with linespoints lt rgb "red" linewidth 2 pointtype 6 pointsize 0.5 title "d = 3",\
    "" using 1:9:($9*(1-$10)):($9*(1+$10)) with errorbars lt rgb "red" linewidth 2 pointtype 6 pointsize 0.5 notitle,\
    "d_5.txt" using 1:9 with linespoints lt rgb "blue" linewidth 2 pointtype 2 pointsize 0.5 title "d = 5",\
    "" using 1:9:($9*(1-$10)):($9*(1+$10)) with errorbars lt rgb "blue" linewidth 2 pointtype 2 pointsize 0.5 notitle,\
    "d_7.txt" using 1:9 with linespoints lt rgb "green" linewidth 2 pointtype 2 pointsize 0.5 title "d = 7",\
    "" using 1:9:($9*(1-$10)):($9*(1+$10)) with errorbars lt rgb "green" linewidth 2 pointtype 2 pointsize 0.5 notitle,\
    "d_9.txt" using 1:9 with linespoints lt rgb "yellow" linewidth 2 pointtype 2 pointsize 0.5 title "d = 9",\
    "" using 1:9:($9*(1-$10)):($9*(1+$10)) with errorbars lt rgb "yellow" linewidth 2 pointtype 2 pointsize 0.5 notitle,\
    "d_11.txt" using 1:9 with linespoints lt rgb "purple" linewidth 2 pointtype 2 pointsize 0.5 title "d = 11",\
    "" using 1:9:($9*(1-$10)):($9*(1+$10)) with errorbars lt rgb "purple" linewidth 2 pointtype 2 pointsize 0.5 notitle,\
    "d_13.txt" using 1:9 with linespoints lt rgb "orange" linewidth 2 pointtype 2 pointsize 0.5 title "d = 13",\
    "" using 1:9:($9*(1-$10)):($9*(1+$10)) with errorbars lt rgb "orange" linewidth 2 pointtype 2 pointsize 0.5 notitle

system("ps2pdf -dEPSCrop init_measurement_error_0.01_fbench_fake.eps init_measurement_error_0.01_fbench_fake.pdf")

# set size 1,0.75
# set output "init_measurement_error_0.01_fbench_fake_w.eps"
# replot
# system("ps2pdf -dEPSCrop init_measurement_error_0.01_fbench_fake_w.eps init_measurement_error_0.01_fbench_fake_w.pdf")

# set size 1,0.6
# set output "init_measurement_error_0.01_fbench_fake_w_w.eps"
# replot
# system("ps2pdf -dEPSCrop init_measurement_error_0.01_fbench_fake_w_w.eps init_measurement_error_0.01_fbench_fake_w_w.pdf")
