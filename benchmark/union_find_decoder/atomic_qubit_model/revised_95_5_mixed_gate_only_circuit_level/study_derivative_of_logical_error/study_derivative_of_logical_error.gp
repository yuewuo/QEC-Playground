set terminal postscript eps color "Arial, 28"
set xlabel "Error Rate (p)" font "Arial, 28"
set ylabel "Derivative ({/Symbol D} ln(p_L) / {/Symbol D} ln(p))" font "Arial, 28"
set grid ytics
set size 1,1

set logscale x
set xrange [0.00005:0.05]
set xtics ("5e-5" 0.00005, "5e-4" 0.0005, "5e-3" 0.005, "5e-2" 0.05)
# set logscale y
# python -c "for i in range(0, 11): print('\'%d\' %d' % (i, i), end=', ')"
set ytics ('1' 1, '2' 2, '3' 3, '4' 4, '5' 5, '6' 6, '7' 7, '8' 8)
set yrange [1:8]
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder

set output "study_derivative_of_logical_error.eps"

set title "Derivative of Logical Error"

plot "mixed_d_3.txt" using 1:10 with linespoints lt rgb "red" linewidth 3 pointtype 6 pointsize 1 title "mixed d = 3",\
    "" using 1:10:($10-$11):($10+$11) with errorbars lt rgb "red" linewidth 3 pointtype 6 pointsize 1 notitle,\
    "pure_pauli_d_3.txt" using 1:10 with linespoints lt rgb "light-red" linewidth 3 pointtype 2 pointsize 1 title "pure Pauli d = 3",\
    "" using 1:10:($10-$11):($10+$11) with errorbars lt rgb "light-red" linewidth 3 pointtype 2 pointsize 1 notitle,\
    "mixed_d_5.txt" using 1:10 with linespoints lt rgb "blue" linewidth 3 pointtype 6 pointsize 1 title "mixed d = 5",\
    "" using 1:10:($10-$11):($10+$11) with errorbars lt rgb "blue" linewidth 3 pointtype 6 pointsize 1 notitle,\
    "pure_pauli_d_5.txt" using 1:10 with linespoints lt rgb "light-blue" linewidth 3 pointtype 2 pointsize 1 title "pure Pauli d = 5",\
    "" using 1:10:($10-$11):($10+$11) with errorbars lt rgb "light-blue" linewidth 3 pointtype 2 pointsize 1 notitle,\
    "mixed_d_7.txt" using 1:10 with linespoints lt rgb "green" linewidth 3 pointtype 6 pointsize 1 title "mixed d = 7",\
    "" using 1:10:($10-$11):($10+$11) with errorbars lt rgb "green" linewidth 3 pointtype 6 pointsize 1 notitle,\
    "pure_pauli_d_7.txt" using 1:10 with linespoints lt rgb "light-green" linewidth 3 pointtype 2 pointsize 1 title "pure Pauli d = 7",\
    "" using 1:10:($10-$11):($10+$11) with errorbars lt rgb "light-green" linewidth 3 pointtype 2 pointsize 1 notitle,\
    "mixed_d_9.txt" using 1:10 with linespoints lt rgb "orange" linewidth 3 pointtype 6 pointsize 1 title "mixed d = 9",\
    "" using 1:10:($10-$11):($10+$11) with errorbars lt rgb "orange" linewidth 3 pointtype 6 pointsize 1 notitle,\
    "pure_pauli_d_9.txt" using 1:10 with linespoints lt rgb "yellow" linewidth 3 pointtype 2 pointsize 1 title "pure Pauli d = 9",\
    "" using 1:10:($10-$11):($10+$11) with errorbars lt rgb "yellow" linewidth 3 pointtype 2 pointsize 1 notitle

system("ps2pdf -dEPSCrop study_derivative_of_logical_error.eps study_derivative_of_logical_error.pdf")

# set size 1,0.75
# set output "study_derivative_of_logical_error_w.eps"
# replot
# system("ps2pdf -dEPSCrop study_derivative_of_logical_error_w.eps study_derivative_of_logical_error_w.pdf")

# set size 1,0.6
# set output "study_derivative_of_logical_error_w_w.eps"
# replot
# system("ps2pdf -dEPSCrop study_derivative_of_logical_error_w_w.eps study_derivative_of_logical_error_w_w.pdf")
