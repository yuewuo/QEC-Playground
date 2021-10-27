set terminal postscript eps color "Arial, 22"
set xlabel "Noise Bias {/Symbol h} = p_Z / (p_X + p_Y)" font "Arial, 22"
set ylabel "Logical Error Rate (p_L)" font "Arial, 22"
set grid ytics
set size 1,1

set logscale x
set xrange [0.5:100000]
set xtics ('0.5' 0.5, '1' 1, '1e1' 10, '1e2' 100, '1e3' 1000, '1e4' 10000, '{/Symbol \245}' 100000)
set logscale y
set ytics ("0.001" 0.001, "0.003" 0.003, "0.01" 0.01, "0.03" 0.03, "0.1" 0.1)
set yrange [0.001:0.2]
set key outside horizontal top center font "Arial, 22"

set style fill transparent solid 0.2 noborder

# set title "XZZX Surface Code p = 0.1"

set output "code_capacity_noise_model_CSS.eps"

plot "MWPM_d11_p0.1.txt" using 1:6 with linespoints lt rgb "red" linewidth 4 pointtype 6 pointsize 1.3 title "MWPM, d = 11",\
    "" using 1:6:($6*(1-$7)):($6*(1+$7)) with errorbars lt rgb "red" linewidth 4 pointtype 6 pointsize 1.3 notitle,\
    "UF_d11_p0.1.txt" using 1:6 with linespoints lt rgb "blue" linewidth 4 pointtype 2 pointsize 1.3 title "UnionFind, d = 11",\
    "" using 1:6:($6*(1-$7)):($6*(1+$7)) with errorbars lt rgb "blue" linewidth 4 pointtype 2 pointsize 1.3 notitle,\
    "MWPM_d13_p0.1.txt" using 1:6 with linespoints lt rgb "orange" linewidth 4 pointtype 6 pointsize 1.3 title "MWPM, d = 13",\
    "" using 1:6:($6*(1-$7)):($6*(1+$7)) with errorbars lt rgb "orange" linewidth 4 pointtype 6 pointsize 1.3 notitle,\
    "UF_d13_p0.1.txt" using 1:6 with linespoints lt rgb "skyblue" linewidth 4 pointtype 2 pointsize 1.3 title "UnionFind, d = 13",\
    "" using 1:6:($6*(1-$7)):($6*(1+$7)) with errorbars lt rgb "skyblue" linewidth 4 pointtype 2 pointsize 1.3 notitle

set output '|ps2pdf -dEPSCrop code_capacity_noise_model_CSS.eps code_capacity_noise_model_CSS.pdf'
replot

# set size 1,0.75
# set output "code_capacity_noise_model_CSS_w.eps"
# replot
# set output '|ps2pdf -dEPSCrop code_capacity_noise_model_CSS_w.eps code_capacity_noise_model_CSS_w.pdf'
# replot

# set size 1,0.6
# set output "code_capacity_noise_model_CSS_w_w.eps"
# replot
# set output '|ps2pdf -dEPSCrop code_capacity_noise_model_CSS_w_w.eps code_capacity_noise_model_CSS_w_w.pdf'
# replot