set terminal postscript eps color "Arial, 28"
set xlabel "Noise Bias {/Symbol z}" font "Arial, 28"
set ylabel "Logical Error Rate (p_L)" font "Arial, 28"
# set grid ytics
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
# labels
# python -c "for i in range(2, 10): print('\'\' %.4f' % (0.0001 * i), end=', ')"
# python -c "for i in range(2, 10): print('\'\' %.3f' % (0.001 * i), end=', ')"
# python -c "for i in range(2, 10): print('\'\' %.2f' % (0.01 * i), end=', ')"
# python -c "for i in range(2, 10): print('\'\' %.1f' % (0.1 * i), end=', ')"
set ytics ("0.0001" 0.0001, '' 0.0002, '' 0.0003, '' 0.0004, '' 0.0005, '' 0.0006, '' 0.0007, '' 0.0008, '' 0.0009,\
"0.001" 0.001, '' 0.002, '' 0.003, '' 0.004, '' 0.005, '' 0.006, '' 0.007, '' 0.008, '' 0.009,\
"0.01" 0.01, '' 0.02, '' 0.03, '' 0.04, '' 0.05, '' 0.06, '' 0.07, '' 0.08, '' 0.09,\
"0.1" 0.1, '' 0.2, '' 0.3, '' 0.4, '' 0.5, '' 0.6, '' 0.7, '' 0.8, '' 0.9)
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder

set title "XZZX Code, p_{CX} = 0.006 = p_Z * (2 + 12/{/Symbol z})"

set output "compare_with_MWPM.eps"

# to remove legend (because I don't know how to plot it just like Fig.7 in arXiv2104.09539v1)
set nokey

plot "MWPM_d3.txt" using 1:7 with linespoints lt rgb "#e41a1c" linewidth 3 pointtype 7 pointsize 1 title "MWPM 3x9x9",\
    "" using 1:7:($7-$7*$9):($7+$7*$9) with errorbars lt rgb "#e41a1c" linewidth 3 pointtype 7 pointsize 1,\
    "UF_d3.txt" using 1:7 with linespoints lt rgb "#4daf4a" linewidth 3 pointtype 6 pointsize 1 title "UnionFind 3x9x9",\
    "" using 1:7:($7-$7*$9):($7+$7*$9) with errorbars lt rgb "#4daf4a" linewidth 3 pointtype 6 pointsize 1,\
    "MWPM_d5.txt" using 1:7 with linespoints lt rgb "#e41a1c" linewidth 3 pointtype 11 pointsize 1 title "MWPM 5x15x15",\
    "" using 1:7:($7-$7*$9):($7+$7*$9) with errorbars lt rgb "#e41a1c" linewidth 3 pointtype 11 pointsize 1,\
    "UF_d5.txt" using 1:7 with linespoints lt rgb "#4daf4a" linewidth 3 pointtype 10 pointsize 1 title "UnionFind 5x15x15",\
    "" using 1:7:($7-$7*$9):($7+$7*$9) with errorbars lt rgb "#4daf4a" linewidth 3 pointtype 10 pointsize 1,\
    "MWPM_d7.txt" using 1:7 with linespoints lt rgb "#e41a1c" linewidth 3 pointtype 13 pointsize 1 title "MWPM 7x21x21",\
    "" using 1:7:($7-$7*$9):($7+$7*$9) with errorbars lt rgb "#e41a1c" linewidth 3 pointtype 13 pointsize 1,\
    "UF_d7.txt" using 1:7 with linespoints lt rgb "#4daf4a" linewidth 3 pointtype 12 pointsize 1 title "UnionFind 7x21x21",\
    "" using 1:7:($7-$7*$9):($7+$7*$9) with errorbars lt rgb "#4daf4a" linewidth 3 pointtype 12 pointsize 1,\

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
