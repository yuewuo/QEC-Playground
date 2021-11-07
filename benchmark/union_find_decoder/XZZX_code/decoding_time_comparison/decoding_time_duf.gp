set terminal postscript eps color "Arial, 28"
set xlabel "Code Distance (d)" font "Arial, 28"
set ylabel "Decoding Clock Cycles" font "Arial, 28"
# set grid ytics
set size 1,1

set logscale x
set xrange [3:8]
# labels
# python -c "for i in range(3, 9): print('\'%d\' %d' % (i, i), end=', ')"
set xtics ('3' 3, '4' 4, '5' 5, '6' 6, '7' 7, '8' 8)
set logscale y
set yrange [1:10000]
# labels
# python -c "for i in range(0, 5): print('\'%d\' %d' % tuple([1 * (10**i) for j in range(2)]), end=', ')"
set ytics ('1' 1, '10' 10, '100' 100, '1000' 1000, '10000' 10000)
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder

set title "Decoding Time of XZZX Code d x 3d x 3d"

set output "decoding_time_duf.eps"

# to remove legend (because I don't know how to plot it just like Fig.7 in arXiv2104.09539v1)
set nokey

plot "DUF_clock_cycles.txt" using 1:2 with linespoints lt rgb "#e41a1c" linewidth 3 pointtype 7 pointsize 1 title "DUF",\
    "" using 1:2:2:3 with errorbars lt rgb "#e41a1c" linewidth 3 pointtype 7 pointsize 1

system("ps2pdf -dEPSCrop decoding_time_duf.eps decoding_time_duf.pdf")

set size 1,0.75
set output "decoding_time_duf_w.eps"
replot
system("ps2pdf -dEPSCrop decoding_time_duf_w.eps decoding_time_duf_w.pdf")

set size 1,0.6
set output "decoding_time_duf_w_w.eps"
replot
system("ps2pdf -dEPSCrop decoding_time_duf_w_w.eps decoding_time_duf_w_w.pdf")
