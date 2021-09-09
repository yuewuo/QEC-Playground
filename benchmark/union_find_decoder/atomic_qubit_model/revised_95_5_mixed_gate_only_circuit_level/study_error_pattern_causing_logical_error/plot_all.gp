set terminal postscript eps color "Arial, 12"

set output "plot_all.eps"
set size 1,1

set multiplot layout 2,2 title "Ratio of Erasure Errors w/ Logical Error" font "Arial, 24"

set xlabel "Erasure Ratio (#Erasure / (#Erasure + #Pauli))" font "Arial, 12"
set ylabel "% of Total" font "Arial, 12"
set grid ytics

# set logscale x
set xrange [0:1]
# python -c "for i in range(0, 6): print('\'%.1f\' %.1f' % (i/5, i/5), end=', ')"
set xtics ('0' 0, '0.2' 0.2, '0.4' 0.4, '0.6' 0.6, '0.8' 0.8, '1' 1)
# set logscale y
# python -c "for i in range(0, 6): print('\'%.1f\' %.1f' % tuple([i*0.1 for j in range(2)]), end=', ')"
set ytics ('0.0' 0.0, '0.1' 0.1, '0.2' 0.2, '0.3' 0.3, '0.4' 0.4, '0.5' 0.5)
set yrange [0:0.5]
set key outside horizontal top center font "Arial, 10"

set style fill transparent solid 0.2





binwidth = 0.02
set boxwidth binwidth

set title "d = 3"
plot "data_d_3.txt" using 1:3 smooth freq with boxes notitle
set title "d = 5"
plot "data_d_5.txt" using 1:3 smooth freq with boxes notitle
set title "d = 7"
plot "data_d_7.txt" using 1:3 smooth freq with boxes notitle
set title "d = 9"
plot "data_d_9.txt" using 1:3 smooth freq with boxes notitle


unset multiplot

set output '|ps2pdf -dEPSCrop plot_all.eps plot_all.pdf'
replot

# set size 1,0.75
# set output "plot_all_w.eps"
# replot
# set output '|ps2pdf -dEPSCrop plot_all_w.eps plot_all_w.pdf'
# replot

# set size 1,0.6
# set output "plot_all_w_w.eps"
# replot
# set output '|ps2pdf -dEPSCrop plot_all_w_w.eps plot_all_w_w.pdf'
# replot
