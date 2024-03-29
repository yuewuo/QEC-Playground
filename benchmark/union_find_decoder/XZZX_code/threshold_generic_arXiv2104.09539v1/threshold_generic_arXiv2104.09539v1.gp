set terminal postscript eps color "Arial, 28"
set xlabel "p_z" font "Arial, 28"
set ylabel "Logical error rate" font "Arial, 28"
# set grid ytics
set size 1,1.1

# data range:
# python -c "for i in range(13): print('%.4f' % (0.008 + (i-6)*0.0005), end=',')"

# data generating commands:
# biased CX
# RUST_BACKTRACE=full cargo run --release -- tool fault_tolerant_benchmark [4,5,6,7,8] --djs [12,15,18,21,24] [12,15,18,21,24] [0.0050,0.0055,0.0060,0.0065,0.0070,0.0075,0.0080,0.0085,0.0090,0.0095,0.0100,0.0105,0.0110]-p0 -m100000 -e100000 --use_xzzx_code --bias_eta 100 --decoder UF --max_half_weight 10 --noise_model GenericBiasedWithBiasedCX
# standard CX
# RUST_BACKTRACE=full cargo run --release -- tool fault_tolerant_benchmark [4,5,6,7,8] --djs [12,15,18,21,24] [12,15,18,21,24] [0.0050,0.0055,0.0060,0.0065,0.0070,0.0075,0.0080,0.0085,0.0090,0.0095,0.0100,0.0105,0.0110]-p0 -m100000 -e100000 --use_xzzx_code --bias_eta 100 --decoder UF --max_half_weight 10 --noise_model GenericBiasedWithStandardCX


set xrange [0.0045:0.0115]
# labels
# python -c "for i in range(7): print('\'%.3f\' %.3f' % tuple([0.008 + (i-3)*0.001 for j in range(2)]), end=', ')"
set xtics ('0.005' 0.005, '0.006' 0.006, '0.007' 0.007, '0.008' 0.008, '0.009' 0.009, '0.010' 0.010, '0.011' 0.011)
set logscale y
# labels
# python -c "for i in range(2, 10): print('\'\' %.4f' % (0.0001 * i), end=', ')"
# python -c "for i in range(2, 10): print('\'\' %.3f' % (0.001 * i), end=', ')"
# python -c "for i in range(2, 10): print('\'\' %.2f' % (0.01 * i), end=', ')"
# python -c "for i in range(2, 10): print('\'\' %.1f' % (0.1 * i), end=', ')"
set ytics ("10^{-4}" 0.0001, '' 0.0002, '' 0.0003, '' 0.0004, '' 0.0005, '' 0.0006, '' 0.0007, '' 0.0008, '' 0.0009, \
"10^{-3}" 0.001, '' 0.002, '' 0.003, '' 0.004, '' 0.005, '' 0.006, '' 0.007, '' 0.008, '' 0.009, \
"10^{-2}" 0.01, '' 0.02, '' 0.03, '' 0.04, '' 0.05, '' 0.06, '' 0.07, '' 0.08, '' 0.09, \
"10^{-1}" 0.1, '' 0.2, '' 0.3, '' 0.4, '' 0.5, '' 0.6, '' 0.7, '' 0.8, '' 0.9, "10^{0}" 1)
set yrange [0.0001:1]
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder

set title "Generaic Biased Noise Model (UnionFind Decoder)"

set output "threshold_generic_arXiv2104.09539v1.eps"

# to remove legend (because I don't know how to plot it just like Fig.7 in arXiv2104.09539v1)
set nokey

plot "biased_4.txt" using 1:6 with linespoints lt rgb "#e41a1c" linewidth 4 pointtype 7 pointsize 1.5 title "biased 4,12,12",\
    "" using 1:6:($6-$6*$8):($6+$6*$8) with errorbars lt rgb "#e41a1c" linewidth 4 pointtype 7 pointsize 1.5,\
    "biased_5.txt" using 1:6 with linespoints lt rgb "#377eb8" linewidth 4 pointtype 9 pointsize 1.5 title "biased 5,15,15",\
    "" using 1:6:($6-$6*$8):($6+$6*$8) with errorbars lt rgb "#377eb8" linewidth 4 pointtype 9 pointsize 1.5,\
    "biased_6.txt" using 1:6 with linespoints lt rgb "#4daf4a" linewidth 4 pointtype 11 pointsize 1.5 title "biased 6,18,18",\
    "" using 1:6:($6-$6*$8):($6+$6*$8) with errorbars lt rgb "#4daf4a" linewidth 4 pointtype 11 pointsize 1.5,\
    "biased_7.txt" using 1:6 with linespoints lt rgb "#ff7f00" linewidth 4 pointtype 5 pointsize 1.5 title "biased 7,21,21",\
    "" using 1:6:($6-$6*$8):($6+$6*$8) with errorbars lt rgb "#ff7f00" linewidth 4 pointtype 5 pointsize 1.5,\
    "biased_8.txt" using 1:6 with linespoints lt rgb "#984ea3" linewidth 4 pointtype 13 pointsize 1.5 title "biased 8,24,24",\
    "" using 1:6:($6-$6*$8):($6+$6*$8) with errorbars lt rgb "#984ea3" linewidth 4 pointtype 13 pointsize 1.5,\
    "standard_4.txt" using 1:6 with linespoints lt rgb "#e41a1c" linewidth 4 dashtype (1,1) pointtype 6 pointsize 1 title "standard 4,12,12",\
    "" using 1:6:($6-$6*$8):($6+$6*$8) with errorbars lt rgb "#e41a1c" linewidth 4 dashtype (1,1) pointtype 6 pointsize 1,\
    "standard_5.txt" using 1:6 with linespoints lt rgb "#377eb8" linewidth 4 dashtype (1,1) pointtype 8 pointsize 1 title "standard 5,15,15",\
    "" using 1:6:($6-$6*$8):($6+$6*$8) with errorbars lt rgb "#377eb8" linewidth 4 dashtype (1,1) pointtype 8 pointsize 1,\
    "standard_6.txt" using 1:6 with linespoints lt rgb "#4daf4a" linewidth 4 dashtype (1,1) pointtype 10 pointsize 1 title "standard 6,18,18",\
    "" using 1:6:($6-$6*$8):($6+$6*$8) with errorbars lt rgb "#4daf4a" linewidth 4 dashtype (1,1) pointtype 10 pointsize 1,\
    "standard_7.txt" using 1:6 with linespoints lt rgb "#ff7f00" linewidth 4 dashtype (1,1) pointtype 4 pointsize 1 title "standard 7,21,21",\
    "" using 1:6:($6-$6*$8):($6+$6*$8) with errorbars lt rgb "#ff7f00" linewidth 4 dashtype (1,1) pointtype 4 pointsize 1,\
    "standard_8.txt" using 1:6 with linespoints lt rgb "#984ea3" linewidth 4 dashtype (1,1) pointtype 12 pointsize 1 title "standard 8,24,24",\
    "" using 1:6:($6-$6*$8):($6+$6*$8) with errorbars lt rgb "#984ea3" linewidth 4 dashtype (1,1) pointtype 12 pointsize 1

system("ps2pdf -dEPSCrop threshold_generic_arXiv2104.09539v1.eps threshold_generic_arXiv2104.09539v1.pdf")

set size 1,0.75
set output "threshold_generic_arXiv2104.09539v1_w.eps"
replot
system("ps2pdf -dEPSCrop threshold_generic_arXiv2104.09539v1_w.eps threshold_generic_arXiv2104.09539v1_w.pdf")

set size 1,0.6
set output "threshold_generic_arXiv2104.09539v1_w_w.eps"
replot
system("ps2pdf -dEPSCrop threshold_generic_arXiv2104.09539v1_w_w.eps threshold_generic_arXiv2104.09539v1_w_w.pdf")
