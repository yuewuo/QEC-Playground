set terminal postscript eps color "Arial, 28"
set xlabel "Noise Bias {/Symbol z}" font "Arial, 28"
set ylabel "Logical Error Rate (p_L)" font "Arial, 28"
set grid ytics
set size 1,1

set logscale x
set xrange [1:10000]
# labels
# python -c "for i in range(2, 10): print('\'\' %d' % (1 * i), end=', ')"
# python -c "for i in range(2, 10): print('\'\' %d' % (10 * i), end=', ')"
# python -c "for i in range(2, 10): print('\'\' %d' % (100 * i), end=', ')"
set xtics ('1' 1, '' 2, '' 3, '' 4, '' 5, '' 6, '' 7, '' 8, '' 9,\
'10' 10, '' 20, '' 30, '' 40, '' 50, '' 60, '' 70, '' 80, '' 90,\
'100' 100, '' 200, '' 300, '' 400, '' 500, '' 600, '' 700, '' 800, '' 900,\
'1000' 1000, '{/Symbol \245}' 10000)
set logscale y
set yrange [0.0001:0.3]
set ytics ("0.0001" 0.0001, "0.001" 0.001, "0.003" 0.003, "0.03" 0.03, "0.03" 0.03, "0.1" 0.1)
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder

set title "XZZX 5x15x15, p_{CX} = 0.006 = p_Z * (2 + 10/{/Symbol \245})"

set output "compare_with_MWPM.eps"

plot "MWPM.txt" using 1:7 with linespoints lt rgb "red" linewidth 5 pointtype 6 pointsize 1.5 title "MWPM Decoder",\
    "UF.txt" using 1:7 with linespoints lt rgb "blue" linewidth 5 pointtype 2 pointsize 1.5 title "UnionFind Decoder"

set output '|ps2pdf -dEPSCrop compare_with_MWPM.eps compare_with_MWPM.pdf'
replot

set size 1,0.75
set output "compare_with_MWPM_w.eps"
replot
set output '|ps2pdf -dEPSCrop compare_with_MWPM_w.eps compare_with_MWPM_w.pdf'
replot

set size 1,0.6
set output "compare_with_MWPM_w_w.eps"
replot
set output '|ps2pdf -dEPSCrop compare_with_MWPM_w_w.eps compare_with_MWPM_w_w.pdf'
replot
