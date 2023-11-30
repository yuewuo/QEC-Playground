set terminal postscript eps color "Arial, 28"
set xlabel "Depolarizing Error Rate (p)" font "Arial, 28"
set ylabel "Logical Error Rate (p_L)" font "Arial, 28"
set grid ytics
set size 1,1

set logscale x
set xrange [0.00001:0.1]
set xtics ("10^{-5}" 0.00001, "10^{-4}" 0.0001, "10^{-3}" 0.001, "10^{-2}" 0.01, "10^{-1}" 0.1)
set logscale y
set ytics ("10^{-8}" 0.00000001, "10^{-7}" 0.0000001, "10^{-6}" 0.000001, "10^{-5}" 0.00001, "10^{-4}" 0.0001, "10^{-3}" 0.001, "10^{-2}" 0.01, "10^{-1}" 0.1)
set yrange [0.00000001:1]
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder

set output "code_capacity_noise.eps"

plot "code_capacity_normal/d_3.txt" using 1:6 with linespoints lt rgb "red" linewidth 5 pointtype 2 pointsize 1 title "d=3",\
    "code_capacity_normal/d_5.txt" using 1:6 with linespoints lt rgb "blue" linewidth 5 pointtype 2 pointsize 1 title "d=5",\
    "code_capacity_normal/d_7.txt" using 1:6 with linespoints lt rgb "green" linewidth 5 pointtype 2 pointsize 1 title "d=7",\
    "code_capacity_normal/d_9.txt" using 1:6 with linespoints lt rgb "yellow" linewidth 5 pointtype 2 pointsize 1 title "d=9",\
    "code_capacity_normal/d_11.txt" using 1:6 with linespoints lt rgb "purple" linewidth 5 pointtype 2 pointsize 1 title "d=11",\
    "code_capacity_normal/d_13.txt" using 1:6 with linespoints lt rgb "orange" linewidth 5 pointtype 2 pointsize 1 title "d=13",\
    "code_capacity_with_y/d_3.txt" using 1:6 with linespoints lt rgb "red" dashtype 3 linewidth 5 pointtype 2 pointsize 1 title "d=3(Y)",\
    "code_capacity_with_y/d_5.txt" using 1:6 with linespoints lt rgb "blue" dashtype 3 linewidth 5 pointtype 2 pointsize 1 title "d=5(Y)",\
    "code_capacity_with_y/d_7.txt" using 1:6 with linespoints lt rgb "green" dashtype 3 linewidth 5 pointtype 2 pointsize 1 title "d=7(Y)",\
    "code_capacity_with_y/d_9.txt" using 1:6 with linespoints lt rgb "yellow" dashtype 3 linewidth 5 pointtype 2 pointsize 1 title "d=9(Y)",\
    "code_capacity_with_y/d_11.txt" using 1:6 with linespoints lt rgb "purple" dashtype 3 linewidth 5 pointtype 2 pointsize 1 title "d=11(Y)",\
    "code_capacity_with_y/d_13.txt" using 1:6 with linespoints lt rgb "orange" dashtype 3 linewidth 5 pointtype 2 pointsize 1 title "d=13(Y)"

system("ps2pdf -dEPSCrop code_capacity_noise.eps code_capacity_noise.pdf")
