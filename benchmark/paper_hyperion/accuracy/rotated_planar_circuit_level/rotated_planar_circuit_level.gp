set terminal postscript eps color "Arial, 20"
set title "Circuit-Level Noise Rotated Surface Code"
set xlabel "Depolarizing Error Rate (p)" font "Arial, 20"
set ylabel "Logical Error Rate (p_L)" font "Arial, 20"
set grid ytics
set size 1,1.2

set logscale x
set xrange [0.00001:0.1]
set xtics ("10^{-5}" 0.00001, "10^{-4}" 0.0001, "10^{-3}" 0.001, "10^{-2}" 0.01, "10^{-1}" 0.1)
set logscale y
set ytics ("10^{-8}" 0.00000001, "10^{-7}" 0.0000001, "10^{-6}" 0.000001, "10^{-5}" 0.00001, "10^{-4}" 0.0001, "10^{-3}" 0.001, "10^{-2}" 0.01, "10^{-1}" 0.1)
set yrange [0.00000001:1]
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder

set output "rotated_planar_circuit_level.eps"

plot "mwpm/d_3.txt" using 1:6 with linespoints lt rgb "red" linewidth 5 pointtype 2 pointsize 1 notitle,\
    "mwpm/d_5.txt" using 1:6 with linespoints lt rgb "blue" linewidth 5 pointtype 2 pointsize 1 notitle,\
    "mwpm/d_7.txt" using 1:6 with linespoints lt rgb "green" linewidth 5 pointtype 2 pointsize 1 notitle,\
    "mwpm/d_9.txt" using 1:6 with linespoints lt rgb "yellow" linewidth 5 pointtype 2 pointsize 1 notitle,\
    "mwpm/d_11.txt" using 1:6 with linespoints lt rgb "purple" linewidth 5 pointtype 2 pointsize 1 notitle,\
    "mwpm/d_13.txt" using 1:6 with linespoints lt rgb "orange" linewidth 5 pointtype 2 pointsize 1 notitle,\
    "hyperUF/d_3.txt" using 1:6 with linespoints lt rgb "red" dashtype 3 linewidth 5 pointtype 2 pointsize 1 title "d=3(UF)",\
    "hyperUF/d_5.txt" using 1:6 with linespoints lt rgb "blue" dashtype 3 linewidth 5 pointtype 2 pointsize 1 title "d=5(UF)",\
    "hyperUF/d_7.txt" using 1:6 with linespoints lt rgb "green" dashtype 3 linewidth 5 pointtype 2 pointsize 1 title "d=7(UF)",\
    "hyperUF/d_9.txt" using 1:6 with linespoints lt rgb "yellow" dashtype 3 linewidth 5 pointtype 2 pointsize 1 title "d=9(UF)",\
    "hyperUF/d_11.txt" using 1:6 with linespoints lt rgb "purple" dashtype 3 linewidth 5 pointtype 2 pointsize 1 title "d=11(UF)",\
    "hyperUF/d_13.txt" using 1:6 with linespoints lt rgb "orange" dashtype 3 linewidth 5 pointtype 2 pointsize 1 title "d=13(UF)",\
    "hyperion/d_3.txt" using 1:6 with linespoints lt rgb "red" linewidth 5 pointtype 4 pointsize 1 title "d=3",\
    "hyperion/d_5.txt" using 1:6 with linespoints lt rgb "blue" linewidth 5 pointtype 4 pointsize 1 title "d=5",\
    "hyperion/d_7.txt" using 1:6 with linespoints lt rgb "green" linewidth 5 pointtype 4 pointsize 1 title "d=7",\
    "hyperion/d_9.txt" using 1:6 with linespoints lt rgb "yellow" linewidth 5 pointtype 4 pointsize 1 title "d=9",\
    "hyperion/d_11.txt" using 1:6 with linespoints lt rgb "purple" linewidth 5 pointtype 4 pointsize 1 title "d=11",\
    "hyperion/d_13.txt" using 1:6 with linespoints lt rgb "orange" linewidth 5 pointtype 4 pointsize 1 title "d=13"

system("ps2pdf -dEPSCrop rotated_planar_circuit_level.eps rotated_planar_circuit_level.pdf")
