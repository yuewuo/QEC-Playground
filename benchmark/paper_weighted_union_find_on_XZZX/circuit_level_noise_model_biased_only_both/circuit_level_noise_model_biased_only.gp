set terminal postscript eps color "Arial, 22"
set xlabel "Physical error rate (p_Z)" font "Arial, 22"
set ylabel "Logical error rate (p_L)" font "Arial, 22"
# set grid ytics
set size 1,1.1
set encoding utf8

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
set yrange [0.0001:0.5]
set key outside horizontal top center font "Arial, 22"

set style fill transparent solid 0.2 noborder

# set title "Generaic Biased Noise Model (UnionFind Decoder)"

set output "circuit_level_noise_model_biased_only.eps"

# plot legends just like Fig.7 in arXiv2104.09539v1
set key at graph 0.67, 0.3
set key vertical
set key samplen 3
set key maxrows 5
set label "Union-Find" at graph 0.56, 0.35
set label "MWPM" at graph 0.81, 0.35
set object 1 rect from graph 0.54,0.4 to graph 0.965,0.03 lw 1.5

plot \
    NaN with linespoints lt rgb "#f6baba" linewidth 4 dashtype (1,1) pointtype 6 pointsize 1.5 title " ",\
    NaN with linespoints lt rgb "#c3d8e9" linewidth 4 dashtype (1,1) pointtype 8 pointsize 1.5 title " ",\
    NaN with linespoints lt rgb "#c9e7c8" linewidth 4 dashtype (1,1) pointtype 10 pointsize 1.5 title " ",\
    NaN with linespoints lt rgb "#ffd8b2" linewidth 4 dashtype (1,1) pointtype 4 pointsize 1.5 title " ",\
    NaN with linespoints lt rgb "#e0c9e3" linewidth 4 dashtype (1,1) pointtype 12 pointsize 1.5 title " " ,\
    NaN with linespoints lt rgb "#e41a1c" linewidth 4 pointtype 7 pointsize 1.5 title "4×12×12",\
    NaN with linespoints lt rgb "#377eb8" linewidth 4 pointtype 9 pointsize 1.5 title "5×15×15",\
    NaN with linespoints lt rgb "#4daf4a" linewidth 4 pointtype 11 pointsize 1.5 title "6×18×18",\
    NaN with linespoints lt rgb "#ff7f00" linewidth 4 pointtype 5 pointsize 1.5 title "7×21×21",\
    NaN with linespoints lt rgb "#984ea3" linewidth 4 pointtype 13 pointsize 1.5 title "8×24×24" ,\
    "biased_4.txt" using 1:6 with linespoints lt rgb "#f6baba" linewidth 4 dashtype (1,1) pointtype 7 pointsize 1.5 notitle "UF 4,12,12",\
    "" using 1:6:($6-$6*$8):($6+$6*$8) with errorbars lt rgb "#f6baba" linewidth 4 dashtype (1,1) pointtype 7 pointsize 1.5 notitle,\
    "biased_5.txt" using 1:6 with linespoints lt rgb "#c3d8e9" linewidth 4 dashtype (1,1) pointtype 9 pointsize 1.5 notitle "UF 5,15,15",\
    "" using 1:6:($6-$6*$8):($6+$6*$8) with errorbars lt rgb "#c3d8e9" linewidth 4 dashtype (1,1) pointtype 9 pointsize 1.5 notitle,\
    "biased_6.txt" using 1:6 with linespoints lt rgb "#c9e7c8" linewidth 4 dashtype (1,1) pointtype 11 pointsize 1.5 notitle "UF 6,18,18",\
    "" using 1:6:($6-$6*$8):($6+$6*$8) with errorbars lt rgb "#c9e7c8" linewidth 4 dashtype (1,1) pointtype 11 pointsize 1.5 notitle,\
    "biased_7.txt" using 1:6 with linespoints lt rgb "#ffd8b2" linewidth 4 dashtype (1,1) pointtype 5 pointsize 1.5 notitle "UF 7,21,21",\
    "" using 1:6:($6-$6*$8):($6+$6*$8) with errorbars lt rgb "#ffd8b2" linewidth 4 dashtype (1,1) pointtype 5 pointsize 1.5 notitle,\
    "biased_8.txt" using 1:6 with linespoints lt rgb "#e0c9e3" linewidth 4 dashtype (1,1) pointtype 13 pointsize 1.5 notitle "UF 8,24,24",\
    "" using 1:6:($6-$6*$8):($6+$6*$8) with errorbars lt rgb "#e0c9e3" linewidth 4 dashtype (1,1) pointtype 13 pointsize 1.5 notitle,\
    "biased_4_MWPM.txt" using 1:6 with linespoints lt rgb "#e41a1c" linewidth 4 pointtype 6 pointsize 1 notitle "MWPM 4,12,12",\
    "" using 1:6:($6-$6*$8):($6+$6*$8) with errorbars lt rgb "#e41a1c" linewidth 4 pointtype 6 pointsize 1 notitle,\
    "biased_5_MWPM.txt" using 1:6 with linespoints lt rgb "#377eb8" linewidth 4 pointtype 8 pointsize 1 notitle "MWPM 5,15,15",\
    "" using 1:6:($6-$6*$8):($6+$6*$8) with errorbars lt rgb "#377eb8" linewidth 4 pointtype 8 pointsize 1 notitle,\
    "biased_6_MWPM.txt" using 1:6 with linespoints lt rgb "#4daf4a" linewidth 4 pointtype 10 pointsize 1 notitle "MWPM 6,18,18",\
    "" using 1:6:($6-$6*$8):($6+$6*$8) with errorbars lt rgb "#4daf4a" linewidth 4 pointtype 10 pointsize 1 notitle,\
    "biased_7_MWPM.txt" using 1:6 with linespoints lt rgb "#ff7f00" linewidth 4 pointtype 4 pointsize 1 notitle "MWPM 7,21,21",\
    "" using 1:6:($6-$6*$8):($6+$6*$8) with errorbars lt rgb "#ff7f00" linewidth 4 pointtype 4 pointsize 1 notitle,\
    "biased_8_MWPM.txt" using 1:6 with linespoints lt rgb "#984ea3" linewidth 4 pointtype 12 pointsize 1 notitle "MWPM 8,24,24",\
    "" using 1:6:($6-$6*$8):($6+$6*$8) with errorbars lt rgb "#984ea3" linewidth 4 pointtype 12 pointsize 1 notitle

system("ps2pdf -dEPSCrop circuit_level_noise_model_biased_only.eps circuit_level_noise_model_biased_only.pdf")

# set size 1,0.75
# set output "circuit_level_noise_model_biased_only_w.eps"
# replot
# system("ps2pdf -dEPSCrop circuit_level_noise_model_biased_only_w.eps circuit_level_noise_model_biased_only_w.pdf")

# set size 1,0.6
# set output "circuit_level_noise_model_biased_only_w_w.eps"
# replot
# system("ps2pdf -dEPSCrop circuit_level_noise_model_biased_only_w_w.eps circuit_level_noise_model_biased_only_w_w.pdf")
