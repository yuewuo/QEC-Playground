set terminal postscript eps color "Arial, 28"
set xlabel "Noise Bias {/Symbol z}" font "Arial, 28"
set ylabel "Threshold p_{CX}" font "Arial, 28"
# set grid ytics
set size 1,1.1

# data range:
# python -c "for i in range(13): print('%.4f' % (0.008 + (i-6)*0.0005), end=',')"

set logscale x
set xrange [1:10000]
# labels
# python -c "for i in range(2, 10): print('\'\' %d' % tuple([i * 1 for j in range(1)]), end=', ')"
# python -c "for i in range(2, 10): print('\'\' %d' % tuple([i * 10 for j in range(1)]), end=', ')"
# python -c "for i in range(2, 10): print('\'\' %d' % tuple([i * 100 for j in range(1)]), end=', ')"
set xtics ('1' 1, '' 2, '' 3, '' 4, '' 5, '' 6, '' 7, '' 8, '' 9,\
'10' 10, '' 20, '' 30, '' 40, '' 50, '' 60, '' 70, '' 80, '' 90,\
'100' 100, '' 200, '' 300, '' 400, '' 500, '' 600, '' 700, '' 800, '' 900,\
'1000' 1000, '+inf' 10000)
set logscale y
# labels
# python -c "for i in range(2, 10): print('\'\' %.4f' % (0.0001 * i), end=', ')"
# python -c "for i in range(2, 10): print('\'\' %.3f' % (0.001 * i), end=', ')"
# python -c "for i in range(2, 10): print('\'\' %.2f' % (0.01 * i), end=', ')"
# python -c "for i in range(2, 10): print('\'\' %.1f' % (0.1 * i), end=', ')"
set ytics ("10^{-4}" 0.0001, '' 0.0002, '' 0.0003, '' 0.0004, '' 0.0005, '' 0.0006, '' 0.0007, '' 0.0008, '' 0.0009, \
"10^{-3}" 0.001, '' 0.002, '' 0.003, '' 0.004, '' 0.005, '' 0.006, '' 0.007, '' 0.008, '' 0.009, \
"10^{-2}" 0.01, '' 0.02, '' 0.03, '' 0.04, '' 0.05, '' 0.06, '' 0.07, '' 0.08, '' 0.09, \
"10^{-1}" 0.1)
set yrange [0.0001:0.1]
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder

set title "Generaic Biased Noise Model"

set output "threshold_with_zeta_biased_pcx.eps"

# to remove legend (because I don't know how to plot it just like Fig.7 in arXiv2104.09539v1)
set nokey

plot "biased_MWPM.txt" using 1:($2*(2+12/$1)) with linespoints lt rgb "#e41a1c" linewidth 4 pointtype 7 pointsize 1.5 title "MWPM Decoder",\
    "" using 1:($2*(2+12/$1)):(($2-$2*$3)*(2+12/$1)):(($2+$2*$3)*(2+12/$1)) with errorbars lt rgb "#e41a1c" linewidth 4 pointtype 7 pointsize 1.5,\
    "biased_UF.txt" using 1:($2*(2+12/$1)) with linespoints lt rgb "#e41a1c" linewidth 4 dashtype (1,1) pointtype 6 pointsize 1 title "UnionFind Decoder",\
    "" using 1:($2*(2+12/$1)):(($2-$2*$3)*(2+12/$1)):(($2+$2*$3)*(2+12/$1)) with errorbars lt rgb "#e41a1c" linewidth 4 dashtype (1,1) pointtype 6 pointsize 1

system("ps2pdf -dEPSCrop threshold_with_zeta_biased_pcx.eps threshold_with_zeta_biased_pcx.pdf")

set size 1,0.75
set output "threshold_with_zeta_biased_pcx_w.eps"
replot
system("ps2pdf -dEPSCrop threshold_with_zeta_biased_pcx_w.eps threshold_with_zeta_biased_pcx_w.pdf")

set size 1,0.6
set output "threshold_with_zeta_biased_pcx_w_w.eps"
replot
system("ps2pdf -dEPSCrop threshold_with_zeta_biased_pcx_w_w.eps threshold_with_zeta_biased_pcx_w_w.pdf")
