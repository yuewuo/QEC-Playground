set terminal postscript eps color "Arial, 28"
set xlabel "Erasure Ratio (#Erasure / (#Erasure + #Pauli))" font "Arial, 28"
set ylabel "% of Total" font "Arial, 28"
set grid ytics
set size 1,1

# set logscale x
set xrange [0:1]
# python -c "for i in range(0, 6): print('\'%.1f\' %.1f' % (i/5, i/5), end=', ')"
set xtics ('0' 0, '0.2' 0.2, '0.4' 0.4, '0.6' 0.6, '0.8' 0.8, '1' 1)
# set logscale y
# python -c "for i in range(0, 6): print('\'%.1f\' %.1f' % tuple([i*0.1 for j in range(2)]), end=', ')"
set ytics ('0.0' 0.0, '0.1' 0.1, '0.2' 0.2, '0.3' 0.3, '0.4' 0.4, '0.5' 0.5)
set yrange [0:0.5]
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2

set output "plot_d_7.eps"

set title "Ratio of Erasure Errors w/ Logical Error (d = 7)"

binwidth = 0.02
set boxwidth binwidth

plot "data_d_7.txt" using 1:3 smooth freq with boxes notitle

set output '|ps2pdf -dEPSCrop plot_d_7.eps plot_d_7.pdf'
replot

# set size 1,0.75
# set output "plot_d_7_w.eps"
# replot
# set output '|ps2pdf -dEPSCrop plot_d_7_w.eps plot_d_7_w.pdf'
# replot

# set size 1,0.6
# set output "plot_d_7_w_w.eps"
# replot
# set output '|ps2pdf -dEPSCrop plot_d_7_w_w.eps plot_d_7_w_w.pdf'
# replot
