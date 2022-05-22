set terminal postscript eps color "Arial, 24"
set xlabel "Code distance" font "Arial, 24"
set ylabel "Average Memory Access" font "Arial, 24"
set grid ytics
set size 1,1.1

set logscale x
set xrange [2.9:21]
# print(", ".join([f"'{i}' {i}" for i in range(3,13)]))
set xtics ('3' 3, '4' 4, '5' 5, '6' 6, '8' 8, '10' 10, '12' 12, '16' 16, '20' 20)
set logscale y
# print(", ".join([f"'1e{i}' 1e{i}" for i in range(1, 9)]))
set ytics ('1' 1, '10' 1e1, '1e2' 1e2, '1e3' 1e3, '1e4' 1e4, '1e5' 1e5, '1e6' 1e6, '1e7' 1e7)
set yrange [0.99:1e7]
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder
set key samplen 4
# set key maxrows 2
# set key height 5

set output "code_distance_memory_access_eta_100.eps"

# plot "processed_MWPM.txt" using 1:($9/($1+1)) with linespoints lt rgb "red" linewidth 3 pointtype 7 pointsize 1 title "MWPM 1.1x",\
#     "" using 1:7 with linespoints lt rgb "blue" linewidth 3 pointtype 7 pointsize 1 title "MWPM 2x",\
#     "" using 1:10 with linespoints lt rgb "green" linewidth 3 pointtype 7 pointsize 1 title "MWPM Average",\

intercept0002 = "`tail -n1 processed_UF_0.0002.txt | awk '{print $6}' | head -n1`"+0
intercept0005 = "`tail -n1 processed_UF_0.0005.txt | awk '{print $6}' | head -n1`"+0
intercept001 = "`tail -n1 processed_UF_0.001.txt | awk '{print $6}' | head -n1`"+0
intercept002 = "`tail -n1 processed_UF_0.002.txt | awk '{print $6}' | head -n1`"+0
intercept004 = "`tail -n1 processed_UF_0.004.txt | awk '{print $6}' | head -n1`"+0
intercept008 = "`tail -n1 processed_UF_0.008.txt | awk '{print $6}' | head -n1`"+0

# print intercept0002

# "" using 1:($5):($5-$6):($5+$6) with errorbars lt rgb "red" linewidth 3 pointtype 11 pointsize 1 notitle,\

plot exp(3 * log(x) + intercept0002)/(x+1) notitle with lines dashtype 2 lt rgb "#e41a1c" linewidth 3,\
    exp(3 * log(x) + intercept0005)/(x+1) notitle with lines dashtype 2 lt rgb "#377eb8" linewidth 3,\
    exp(3 * log(x) + intercept001)/(x+1) notitle with lines dashtype 2 lt rgb "#4daf4a" linewidth 3,\
    exp(3 * log(x) + intercept002)/(x+1) notitle with lines dashtype 2 lt rgb "#ff7f00" linewidth 3,\
    exp(3 * log(x) + intercept004)/(x+1) notitle with lines dashtype 2 lt rgb "#984ea3" linewidth 3,\
    exp(3 * log(x) + intercept008)/(x+1) notitle with lines dashtype 2 lt rgb "black" linewidth 3,\
    "processed_UF_0.0002.txt" using 1:($9/($1+1)) with linespoints lt rgb "#e41a1c" linewidth 3 pointtype 7 pointsize 1.3 title "0.0002",\
    "processed_UF_0.0005.txt" using 1:($9/($1+1)) with linespoints lt rgb "#377eb8" linewidth 3 pointtype 9 pointsize 1.3 title "0.0005",\
    "processed_UF_0.001.txt" using 1:($9/($1+1)) with linespoints lt rgb "#4daf4a" linewidth 3 pointtype 11 pointsize 1.3 title "0.001",\
    "processed_UF_0.002.txt" using 1:($9/($1+1)) with linespoints lt rgb "#ff7f00" linewidth 3 pointtype 5 pointsize 1.3 title "0.002",\
    "processed_UF_0.004.txt" using 1:($9/($1+1)) with linespoints lt rgb "#984ea3" linewidth 3 pointtype 13 pointsize 1.3 title "0.004",\
    "processed_UF_0.008.txt" using 1:($9/($1+1)) with linespoints lt rgb "black" linewidth 3 pointtype 7 pointsize 1.3 title "0.008"

system("ps2pdf -dEPSCrop code_distance_memory_access_eta_100.eps code_distance_memory_access_eta_100.pdf")

# set size 1,0.75
# set output "code_distance_memory_access_eta_100_w.eps"
# replot
# system("ps2pdf -dEPSCrop code_distance_memory_access_eta_100_w.eps code_distance_memory_access_eta_100_w.pdf")

# set size 1,0.6
# set output "code_distance_memory_access_eta_100_w_w.eps"
# replot
# system("ps2pdf -dEPSCrop code_distance_memory_access_eta_100_w_w.eps code_distance_memory_access_eta_100_w_w.pdf")
