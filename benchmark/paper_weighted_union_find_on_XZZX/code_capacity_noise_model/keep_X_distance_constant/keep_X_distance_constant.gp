set terminal postscript eps color "Arial, 22"
set xlabel "Code Distance" font "Arial, 22"
set ylabel "Z Logical Error Rate Only (p_{L,Z})" font "Arial, 22"
set grid ytics
set size 1,1

# set logscale x
set xrange [3:29]
set xtics ('3' 3, '11' 11, '19' 19, '27' 27)
set logscale y
set ytics ("0.001" 0.001, "0.003" 0.003, "0.01" 0.01, "0.03" 0.03, "0.1" 0.1, "0.3" 0.3)
set yrange [0.001:0.3]
set key outside horizontal top center font "Arial, 22"

set style fill transparent solid 0.2 noborder

set title "Constant X Distance = 3, Changing Z Distance"

set output "keep_X_distance_constant.eps"

plot "MWPM_p0.2.txt" using 3:6 with linespoints lt rgb "red" linewidth 5 pointtype 6 pointsize 1.5 title "MWPM, p = 0.2",\
    "" using 3:6:($6*(1-$7)):($6*(1+$7)) with errorbars lt rgb "red" linewidth 5 pointtype 6 pointsize 1.5 notitle,\
    "UF_p0.2.txt" using 3:6 with linespoints lt rgb "blue" linewidth 5 pointtype 2 pointsize 1.5 title "UnionFind, p = 0.2",\
    "" using 3:6:($6*(1-$7)):($6*(1+$7)) with errorbars lt rgb "blue" linewidth 5 pointtype 2 pointsize 1.5 notitle,\

set output '|ps2pdf -dEPSCrop keep_X_distance_constant.eps keep_X_distance_constant.pdf'
replot

# set size 1,0.75
# set output "keep_X_distance_constant_w.eps"
# replot
# set output '|ps2pdf -dEPSCrop keep_X_distance_constant_w.eps keep_X_distance_constant_w.pdf'
# replot

# set size 1,0.6
# set output "keep_X_distance_constant_w_w.eps"
# replot
# set output '|ps2pdf -dEPSCrop keep_X_distance_constant_w_w.eps keep_X_distance_constant_w_w.pdf'
# replot
