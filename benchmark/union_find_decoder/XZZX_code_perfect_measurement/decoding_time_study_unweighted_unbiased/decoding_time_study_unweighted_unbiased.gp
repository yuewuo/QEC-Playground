set terminal postscript eps color "Arial, 28"
set xlabel "Code Distance (d)" font "Arial, 28"
set ylabel "Avergae Decoding Time (s)" font "Arial, 28"
# set grid ytics
set size 1,1

set logscale x
set xrange [13:63]
# labels
# python -c "for i in [5, 7, 9, 11, 13, 17, 21, 25, 29, 37, 43, 53, 63][::2]: print('\'%d\' %d' % (i, i), end=', ')"
set xtics ('13' 13, '21' 21, '29' 29, '43' 43, '63' 63)
set logscale y
set yrange [0.0000001:0.1]
# labels
# python -c "for i in range(0, 5): print('\'%.0e\' %.0e' % tuple([0.00001 * (10**i) for j in range(2)]), end=', ')"
set ytics ('1e-7' 1e-07, '1e-6' 1e-06, '1e-5' 1e-05, '1e-4' 1e-04, '1e-3' 1e-03, '1e-2' 1e-02)
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder

set title "Decoding Time of XZZX Code d x d x 0"

set output "decoding_time_study_unweighted_unbiased.eps"

plot "time_run_to_stable.txt" using 1:2 with linespoints lt rgb "#e41a1c" linewidth 3 pointtype 7 pointsize 1 title "all",\
    "time_uf_grow.txt" using 1:2 with linespoints lt rgb "#377eb8" linewidth 3 pointtype 9 pointsize 1 title "1. grow",\
    "time_uf_merge.txt" using 1:2 with linespoints lt rgb "#4daf4a" linewidth 3 pointtype 11 pointsize 1 title "2. merge",\
    "time_uf_replace.txt" using 1:2 with linespoints lt rgb "#ff7f00" linewidth 3 pointtype 5 pointsize 1 title "3. replace",\
    "time_uf_update.txt" using 1:2 with linespoints lt rgb "#984ea3" linewidth 3 pointtype 5 pointsize 1 title "4. update",\
    "time_uf_remove.txt" using 1:2 with linespoints lt rgb "#e41a1c" linewidth 3 pointtype 5 pointsize 1 title "5. remove"

set output '|ps2pdf -dEPSCrop decoding_time_study_unweighted_unbiased.eps decoding_time_study_unweighted_unbiased.pdf'
replot

# set size 1,0.75
# set output "decoding_time_study_unweighted_unbiased_w.eps"
# replot
# set output '|ps2pdf -dEPSCrop decoding_time_study_unweighted_unbiased_w.eps decoding_time_study_unweighted_unbiased_w.pdf'
# replot

# set size 1,0.6
# set output "decoding_time_study_unweighted_unbiased_w_w.eps"
# replot
# set output '|ps2pdf -dEPSCrop decoding_time_study_unweighted_unbiased_w_w.eps decoding_time_study_unweighted_unbiased_w_w.pdf'
# replot
