set terminal postscript eps color "Arial, 24"
set xlabel "Code distance" font "Arial, 24"
set ylabel "Avergae Decoding time Per Measurement (s)" font "Arial, 24"
set grid ytics
set size 1,1.1

set logscale x
set xrange [2.9:50]
# print(", ".join([f"'{i}' {i}" for i in range(3,13)]))
set xtics ('3' 3, '4' 4, '5' 5, '6' 6, '8' 8, '10' 10, '12' 12, '16' 16, '20' 20, '24' 24, '32' 32, '40' 40, '48' 48)
set logscale y
# print(", ".join([f"'1e{i}' 1e{i}" for i in range(-4, 2)]))
set ytics ('1e-8' 1e-8, '1e-7' 1e-7, '1e-6' 1e-6, '1e-5' 1e-5, '1e-4' 1e-4, '1e-3' 1e-3, '1e-2' 1e-2, '0.1' 1e-1, '1' 1)
set yrange [1e-8:1]
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder
set key samplen 4
# set key maxrows 2
# set key height 5

set output "code_distance_decoding_time_eta_100_integer.eps"

# plot "processed_MWPM.txt" using 1:($5/150) with linespoints lt rgb "red" linewidth 3 pointtype 7 pointsize 1 title "MWPM 1.1x",\
#     "" using 1:7 with linespoints lt rgb "blue" linewidth 3 pointtype 7 pointsize 1 title "MWPM 2x",\
#     "" using 1:10 with linespoints lt rgb "green" linewidth 3 pointtype 7 pointsize 1 title "MWPM Average",\

intercept0002 = "`tail -n2 processed_UF_integer_0.0002.txt | awk '{print $6}' | head -n1`"+0
intercept0005 = "`tail -n2 processed_UF_integer_0.0005.txt | awk '{print $6}' | head -n1`"+0
intercept001 = "`tail -n2 processed_UF_integer_0.001.txt | awk '{print $6}' | head -n1`"+0
intercept002 = "`tail -n2 processed_UF_integer_0.002.txt | awk '{print $6}' | head -n1`"+0
intercept004 = "`tail -n2 processed_UF_integer_0.004.txt | awk '{print $6}' | head -n1`"+0
intercept008 = "`tail -n2 processed_UF_integer_0.008.txt | awk '{print $6}' | head -n1`"+0

# print intercept0002

# "" using 1:($5):($5-$6):($5+$6) with errorbars lt rgb "red" linewidth 3 pointtype 11 pointsize 1 notitle,\

plot exp(2 * log(x) + intercept0002)/150 notitle with lines dashtype 2 lt rgb "#e41a1c" linewidth 3,\
    exp(2 * log(x) + intercept0005)/150 notitle with lines dashtype 2 lt rgb "#377eb8" linewidth 3,\
    exp(2 * log(x) + intercept001)/150 notitle with lines dashtype 2 lt rgb "#4daf4a" linewidth 3,\
    exp(2 * log(x) + intercept002)/150 notitle with lines dashtype 2 lt rgb "#ff7f00" linewidth 3,\
    exp(2 * log(x) + intercept004)/150 notitle with lines dashtype 2 lt rgb "#984ea3" linewidth 3,\
    exp(2 * log(x) + intercept008)/150 notitle with lines dashtype 2 lt rgb "black" linewidth 3,\
    "processed_UF_integer_0.0002.txt" using 1:($5/150) with linespoints lt rgb "#e41a1c" linewidth 3 pointtype 7 pointsize 1.3 title "0.0002",\
    "processed_UF_integer_0.0005.txt" using 1:($5/150) with linespoints lt rgb "#377eb8" linewidth 3 pointtype 9 pointsize 1.3 title "0.0005",\
    "processed_UF_integer_0.001.txt" using 1:($5/150) with linespoints lt rgb "#4daf4a" linewidth 3 pointtype 11 pointsize 1.3 title "0.001",\
    "processed_UF_integer_0.002.txt" using 1:($5/150) with linespoints lt rgb "#ff7f00" linewidth 3 pointtype 5 pointsize 1.3 title "0.002",\
    "processed_UF_integer_0.004.txt" using 1:($5/150) with linespoints lt rgb "#984ea3" linewidth 3 pointtype 13 pointsize 1.3 title "0.004",\
    "processed_UF_integer_0.008.txt" using 1:($5/150) with linespoints lt rgb "black" linewidth 3 pointtype 7 pointsize 1.3 title "0.008"

system("ps2pdf -dEPSCrop code_distance_decoding_time_eta_100_integer.eps code_distance_decoding_time_eta_100_integer.pdf")

# set size 1,0.75
# set output "code_distance_decoding_time_eta_100_integer_w.eps"
# replot
# system("ps2pdf -dEPSCrop code_distance_decoding_time_eta_100_integer_w.eps code_distance_decoding_time_eta_100_integer_w.pdf")

# set size 1,0.6
# set output "code_distance_decoding_time_eta_100_integer_w_w.eps"
# replot
# system("ps2pdf -dEPSCrop code_distance_decoding_time_eta_100_integer_w_w.eps code_distance_decoding_time_eta_100_integer_w_w.pdf")
