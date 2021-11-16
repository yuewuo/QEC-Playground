set terminal postscript eps color "Arial, 28"
set xlabel "Code Distance (d)" font "Arial, 28"
set ylabel "Avergae Decoding Time (s)" font "Arial, 28"
# set grid ytics
set size 1,1

set logscale x
set xrange [5:63]
# labels
# python -c "for i in [5, 7, 9, 11, 13, 17, 21, 25, 29, 37, 43, 53, 63][::2]: print('\'%d\' %d' % (i, i), end=', ')"
set xtics ('5' 5, '9' 9, '13' 13, '21' 21, '29' 29, '43' 43, '63' 63)
set logscale y
set yrange [0.00001:0.1]
# labels
# python -c "for i in range(0, 5): print('\'%.0e\' %.0e' % tuple([0.00001 * (10**i) for j in range(2)]), end=', ')"
set ytics ('1e-05' 1e-05, '1e-04' 1e-04, '1e-03' 1e-03, '1e-02' 1e-02, '1e-01' 1e-01)
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder

set title "Decoding Time of XZZX Code d x d x 0"

set output "decoding_time_comparison_compare_low_p_high_p.eps"

plot "decoding_time_p_0.01.txt" using 1:2 with linespoints lt rgb "#e41a1c" linewidth 3 pointtype 7 pointsize 1 title "p = 0.01",\
    "decoding_time_p_0.03.txt" using 1:2 with linespoints lt rgb "#377eb8" linewidth 3 pointtype 9 pointsize 1 title "p = 0.03",\
    "decoding_time_p_0.1.txt" using 1:2 with linespoints lt rgb "#4daf4a" linewidth 3 pointtype 11 pointsize 1 title "p = 0.1",\
    "decoding_time_p_0.3.txt" using 1:2 with linespoints lt rgb "#ff7f00" linewidth 3 pointtype 5 pointsize 1 title "p = 0.3"

system("ps2pdf -dEPSCrop decoding_time_comparison_compare_low_p_high_p.eps decoding_time_comparison_compare_low_p_high_p.pdf")

set size 1,0.75
set output "decoding_time_comparison_compare_low_p_high_p_w.eps"
replot
system("ps2pdf -dEPSCrop decoding_time_comparison_compare_low_p_high_p_w.eps decoding_time_comparison_compare_low_p_high_p_w.pdf")

set size 1,0.6
set output "decoding_time_comparison_compare_low_p_high_p_w_w.eps"
replot
system("ps2pdf -dEPSCrop decoding_time_comparison_compare_low_p_high_p_w_w.eps decoding_time_comparison_compare_low_p_high_p_w_w.pdf")
