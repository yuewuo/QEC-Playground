set terminal postscript eps color "Arial, 28"
set xlabel "Code Distance (d)" font "Arial, 28"
set ylabel "Decoding Time (s)" font "Arial, 28"
# set grid ytics
set size 1,1

set logscale x
set xrange [3:8]
# labels
# python -c "for i in range(3, 9): print('\'%d\' %d' % (i, i), end=', ')"
set xtics ('3' 3, '4' 4, '5' 5, '6' 6, '7' 7, '8' 8)
set logscale y
set yrange [0.0001:100]
# labels
# python -c "for i in range(0, 7): print('\'%.0e\' %.0e' % tuple([0.0001 * (10**i) for j in range(2)]), end=', ')"
set ytics ('1e-4' 1e-04, '1e-3' 1e-03, '1e-2' 1e-02, '1e-1' 1e-01, '1' 1e+00, '10' 1e+01, '100' 1e+02)
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder

set title "Decoding Time of XZZX Code d x 3d x 3d"

set output "decoding_time_comparison.eps"

# to remove legend (because I don't know how to plot it just like Fig.7 in arXiv2104.09539v1)
set nokey

plot "MWPM_time_blossom_v_prepare_graph.txt" using 1:2 with linespoints lt rgb "#e41a1c" linewidth 3 pointtype 7 pointsize 1 title "MWPM",\
    "" using 1:2:2:3 with errorbars lt rgb "#e41a1c" linewidth 3 pointtype 7 pointsize 1,\
    "UF_time_run_to_stable.txt" using 1:2 with linespoints lt rgb "#4daf4a" linewidth 3 pointtype 6 pointsize 1 title "UnionFind",\
    "" using 1:2:2:3 with errorbars lt rgb "#4daf4a" linewidth 3 pointtype 6 pointsize 1

set output '|ps2pdf -dEPSCrop decoding_time_comparison.eps decoding_time_comparison.pdf'
replot

set size 1,0.75
set output "decoding_time_comparison_w.eps"
replot
set output '|ps2pdf -dEPSCrop decoding_time_comparison_w.eps decoding_time_comparison_w.pdf'
replot

set size 1,0.6
set output "decoding_time_comparison_w_w.eps"
replot
set output '|ps2pdf -dEPSCrop decoding_time_comparison_w_w.eps decoding_time_comparison_w_w.pdf'
replot
