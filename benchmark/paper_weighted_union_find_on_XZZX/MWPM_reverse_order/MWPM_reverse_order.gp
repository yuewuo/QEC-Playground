set terminal postscript eps color "Arial, 22"
set xlabel "Physical error rate (p)" font "Arial, 22"
set ylabel "Logical error rate (p_L)" font "Arial, 22"
# set grid ytics
set size 1,1.1
set encoding utf8

set xrange [0.09:0.10]
# labels
# python3 -c "for i in range(6): print('\'%.3f\' %.3f' % tuple([0.09 + i*0.002 for j in range(2)]), end=', ')"
set xtics ('0.090' 0.090, '0.092' 0.092, '0.094' 0.094, '0.096' 0.096, '0.098' 0.098, '0.100' 0.100)
set logscale y
# labels
# python -c "for i in range(2, 10): print('\'\' %.4f' % (0.0001 * i), end=', ')"
# python -c "for i in range(2, 10): print('\'\' %.3f' % (0.001 * i), end=', ')"
# python -c "for i in range(2, 10): print('\'\' %.2f' % (0.01 * i), end=', ')"
# python -c "for i in range(2, 10): print('\'\' %.1f' % (0.1 * i), end=', ')"
set ytics ("0.01" 0.01, '0.02' 0.02, '0.03' 0.03, '0.04' 0.04, '0.05' 0.05, '0.06' 0.06, '0.07' 0.07, '0.08' 0.08, '0.09' 0.09, \
"0.10" 0.1, '0.11' 0.11, '0.12' 0.12, '0.13' 0.13, '0.14' 0.14, '0.15' 0.15)
set yrange [0.041:0.15]
set key outside horizontal top center font "Arial, 22"

set style fill transparent solid 0.2 noborder

set output "MWPM_reverse_order.eps"

# plot legends just like Fig.7 in arXiv2104.09539v1
set key at graph 0.73, 0.2
set key vertical
set key samplen 3
set key maxrows 5
set label "MWPM 1" at graph 0.59, 0.25
set label "MWPM 2" at graph 0.79, 0.25
set object 1 rect from graph 0.57,0.3 to graph 0.965,0.03 lw 1.5

plot \
    NaN with linespoints lt rgb "#e41a1c" linewidth 4 dashtype (1,1) pointtype 6 pointsize 1.5 title " ",\
    NaN with linespoints lt rgb "#377eb8" linewidth 4 dashtype (1,1) pointtype 8 pointsize 1.5 title " ",\
    NaN with linespoints lt rgb "#4daf4a" linewidth 4 dashtype (1,1) pointtype 10 pointsize 1.5 title " ",\
    NaN with linespoints lt rgb "#e41a1c" linewidth 4 pointtype 7 pointsize 1.5 title "d=3",\
    NaN with linespoints lt rgb "#377eb8" linewidth 4 pointtype 9 pointsize 1.5 title "d=5",\
    NaN with linespoints lt rgb "#4daf4a" linewidth 4 pointtype 11 pointsize 1.5 title "d=7",\
    "normal/d_3.txt" using 1:6 with linespoints lt rgb "#e41a1c" linewidth 4 pointtype 7 pointsize 1.5 notitle "biased 4,12,12",\
    "" using 1:6:($6-$6*$8):($6+$6*$8) with errorbars lt rgb "#e41a1c" linewidth 4 pointtype 7 pointsize 1.5 notitle,\
    "normal/d_5.txt" using 1:6 with linespoints lt rgb "#377eb8" linewidth 4 pointtype 9 pointsize 1.5 notitle "biased 5,15,15",\
    "" using 1:6:($6-$6*$8):($6+$6*$8) with errorbars lt rgb "#377eb8" linewidth 4 pointtype 9 pointsize 1.5 notitle,\
    "normal/d_7.txt" using 1:6 with linespoints lt rgb "#4daf4a" linewidth 4 pointtype 11 pointsize 1.5 notitle "biased 6,18,18",\
    "" using 1:6:($6-$6*$8):($6+$6*$8) with errorbars lt rgb "#4daf4a" linewidth 4 pointtype 11 pointsize 1.5 notitle,\
    "reversed/d_3.txt" using 1:6 with linespoints lt rgb "#e41a1c" linewidth 4 dashtype (1,1) pointtype 6 pointsize 1 notitle "standard 4,12,12",\
    "" using 1:6:($6-$6*$8):($6+$6*$8) with errorbars lt rgb "#e41a1c" linewidth 4 dashtype (1,1) pointtype 6 pointsize 1 notitle,\
    "reversed/d_5.txt" using 1:6 with linespoints lt rgb "#377eb8" linewidth 4 dashtype (1,1) pointtype 8 pointsize 1 notitle "standard 5,15,15",\
    "" using 1:6:($6-$6*$8):($6+$6*$8) with errorbars lt rgb "#377eb8" linewidth 4 dashtype (1,1) pointtype 8 pointsize 1 notitle,\
    "reversed/d_7.txt" using 1:6 with linespoints lt rgb "#4daf4a" linewidth 4 dashtype (1,1) pointtype 10 pointsize 1 notitle "standard 6,18,18",\
    "" using 1:6:($6-$6*$8):($6+$6*$8) with errorbars lt rgb "#4daf4a" linewidth 4 dashtype (1,1) pointtype 10 pointsize 1 notitle

system("ps2pdf -dEPSCrop MWPM_reverse_order.eps MWPM_reverse_order.pdf")

# set size 1,0.75
# set output "MWPM_reverse_order_w.eps"
# replot
# system("ps2pdf -dEPSCrop MWPM_reverse_order_w.eps MWPM_reverse_order_w.pdf")

# set size 1,0.6
# set output "MWPM_reverse_order_w_w.eps"
# replot
# system("ps2pdf -dEPSCrop MWPM_reverse_order_w_w.eps MWPM_reverse_order_w_w.pdf")
