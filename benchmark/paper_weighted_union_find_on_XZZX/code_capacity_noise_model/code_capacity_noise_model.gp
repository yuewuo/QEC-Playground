set terminal postscript eps color "Arial, 28"
set xlabel "{/Symbol h}" font "Arial, 28"
set ylabel "Logical Error Rate (p_L)" font "Arial, 28"
set grid ytics
set size 1,1

set logscale x
set xrange [0.5:100000]
set xtics ('1' 1, '1e1' 10, '1e2' 100, '1e3' 1000, '1e4' 10000, '{/Symbol \245}' 100000)
set logscale y
set ytics ("0.001" 0.001, "0.003" 0.003, "0.01" 0.01, "0.03" 0.03, "0.1" 0.1)
set yrange [0.001:0.1]
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder

set title "XZZX Surface Code p = 0.1"

set output "code_capacity_noise_model.eps"

plot "MWPM_d11_p0.1.txt" using 1:6 with linespoints lt rgb "red" linewidth 5 pointtype 6 pointsize 1.5 title "MWPM, d = 11",\
    "" using 1:6:($6*(1-$7)):($6*(1+$7)) with errorbars lt rgb "red" linewidth 5 pointtype 6 pointsize 1.5 notitle,\
    "UF_d11_p0.1.txt" using 1:6 with linespoints lt rgb "blue" linewidth 5 pointtype 2 pointsize 1.5 title "UnionFind, d = 11",\
    "" using 1:6:($6*(1-$7)):($6*(1+$7)) with errorbars lt rgb "blue" linewidth 5 pointtype 2 pointsize 1.5 notitle,\
    "MWPM_d13_p0.1.txt" using 1:6 with linespoints lt rgb "orange" linewidth 5 pointtype 6 pointsize 1.5 title "MWPM, d = 13",\
    "" using 1:6:($6*(1-$7)):($6*(1+$7)) with errorbars lt rgb "orange" linewidth 5 pointtype 6 pointsize 1.5 notitle,\
    "UF_d13_p0.1.txt" using 1:6 with linespoints lt rgb "skyblue" linewidth 5 pointtype 2 pointsize 1.5 title "UnionFind, d = 13",\
    "" using 1:6:($6*(1-$7)):($6*(1+$7)) with errorbars lt rgb "skyblue" linewidth 5 pointtype 2 pointsize 1.5 notitle

set output '|ps2pdf -dEPSCrop code_capacity_noise_model.eps code_capacity_noise_model.pdf'
replot

# set size 1,0.75
# set output "code_capacity_noise_model_w.eps"
# replot
# set output '|ps2pdf -dEPSCrop code_capacity_noise_model_w.eps code_capacity_noise_model_w.pdf'
# replot

# set size 1,0.6
# set output "code_capacity_noise_model_w_w.eps"
# replot
# set output '|ps2pdf -dEPSCrop code_capacity_noise_model_w_w.eps code_capacity_noise_model_w_w.pdf'
# replot
