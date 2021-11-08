set terminal postscript eps color "Arial, 22"
set xlabel "Code Distance d" font "Arial, 22"
set ylabel "Decoding Time (s)" font "Arial, 22"
set grid ytics
set size 1,1

set logscale x
set xrange [2.5:12.5]
# print(", ".join([f"'{i}' {i}" for i in range(3,13)]))
set xtics ('3' 3, '4' 4, '5' 5, '6' 6, '7' 7, '8' 8, '9' 9, '10' 10, '11' 11, '12' 12)
set logscale y
# print(", ".join([f"'1e{i}' 1e{i}" for i in range(-4, 2)]))
set ytics ('1e-4' 1e-4, '1e-3' 1e-3, '1e-2' 1e-2, '0.1' 1e-1, '1' 1e0, '10' 1e1)
set yrange [3e-5:20]
set key outside horizontal top center font "Arial, 22"

set style fill transparent solid 0.2 noborder
set key samplen 2
set key maxrows 3

# set title "XZZX Surface Code p = 0.07"

set output "code_distance_decoding_time_eta_100.eps"


plot "processed_MWPM.txt" using 1:12 with linespoints lt rgb "red" linewidth 3 pointtype 7 pointsize 1 title "MWPM Max",\
    "" using 1:7 with linespoints lt rgb "blue" linewidth 3 pointtype 7 pointsize 1 title "MWPM 2x",\
    "" using 1:10 with linespoints lt rgb "green" linewidth 3 pointtype 7 pointsize 1 title "MWPM Average",\
    "processed_UF.txt" using 1:12 with linespoints lt rgb "light-red" linewidth 3 pointtype 7 pointsize 1 title "UF Max",\
    "" using 1:7 with linespoints lt rgb "light-blue" linewidth 3 pointtype 7 pointsize 1 title "UF 2x",\
    "" using 1:10 with linespoints lt rgb "light-green" linewidth 3 pointtype 7 pointsize 1 title "UF Average"
    # "" using 1:6:($6*(1-$7)):($6*(1+$7)) with errorbars lt rgb "red" linewidth 3 pointtype 7 pointsize 1 notitle,\
    # "UF_d11_p0.07.txt" using 1:6 with linespoints lt rgb "blue" linewidth 3 pointtype 11 pointsize 1 title "UnionFind, d = 11",\
    # "" using 1:6:($6*(1-$7)):($6*(1+$7)) with errorbars lt rgb "blue" linewidth 3 pointtype 11 pointsize 1 notitle,\
    # "MWPM_d13_p0.07.txt" using 1:6 with linespoints lt rgb "orange" linewidth 3 pointtype 7 pointsize 1 title "MWPM, d = 13",\
    # "" using 1:6:($6*(1-$7)):($6*(1+$7)) with errorbars lt rgb "orange" linewidth 3 pointtype 7 pointsize 1 notitle,\
    # "UF_d13_p0.07.txt" using 1:6 with linespoints lt rgb "skyblue" linewidth 3 pointtype 11 pointsize 1 title "UnionFind, d = 13",\
    # "" using 1:6:($6*(1-$7)):($6*(1+$7)) with errorbars lt rgb "skyblue" linewidth 3 pointtype 11 pointsize 1 notitle

system("ps2pdf -dEPSCrop code_distance_decoding_time_eta_100.eps code_distance_decoding_time_eta_100.pdf")

# set size 1,0.75
# set output "code_distance_decoding_time_eta_100_w.eps"
# replot
# system("ps2pdf -dEPSCrop code_distance_decoding_time_eta_100_w.eps code_distance_decoding_time_eta_100_w.pdf")

# set size 1,0.6
# set output "code_distance_decoding_time_eta_100_w_w.eps"
# replot
# system("ps2pdf -dEPSCrop code_distance_decoding_time_eta_100_w_w.eps code_distance_decoding_time_eta_100_w_w.pdf")
