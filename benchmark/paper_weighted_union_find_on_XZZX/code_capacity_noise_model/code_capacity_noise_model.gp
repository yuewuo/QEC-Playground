set terminal postscript eps color "Arial, 22"
set xlabel "Noise Bias {/Symbol h} = p_Z / (p_X + p_Y)" font "Arial, 22"
set ylabel "Logical Error Rate (p_L)" font "Arial, 22"
set grid ytics
set size 1,1

set logscale x
set xrange [0.5:50000]
set xtics ('0.5' 0.5, '5' 5, '50' 50, '5e2' 500, '5e3' 5000, '{/Symbol \245}' 50000)
set logscale y
set ytics ("1e-4" 0.0001, "3e-4" 0.0003, "1e-3" 0.001, "3e-3" 0.003, "1e-2" 0.01, "3e-2" 0.03)
set yrange [0.0001:0.04]
set key outside horizontal top center font "Arial, 22"

set style fill transparent solid 0.2 noborder
set key samplen 2

# set title "XZZX Surface Code p = 0.07"

set output "code_capacity_noise_model.eps"

plot "MWPM_d11_p0.07.txt" using 1:6 with linespoints lt rgb "red" linewidth 3 pointtype 7 pointsize 1 title "MWPM, d = 11",\
    "" using 1:6:($6*(1-$7)):($6*(1+$7)) with errorbars lt rgb "red" linewidth 3 pointtype 7 pointsize 1 notitle,\
    "UF_d11_p0.07.txt" using 1:6 with linespoints lt rgb "blue" linewidth 3 pointtype 11 pointsize 1 title "UnionFind, d = 11",\
    "" using 1:6:($6*(1-$7)):($6*(1+$7)) with errorbars lt rgb "blue" linewidth 3 pointtype 11 pointsize 1 notitle,\
    "MWPM_d13_p0.07.txt" using 1:6 with linespoints lt rgb "orange" linewidth 3 pointtype 7 pointsize 1 title "MWPM, d = 13",\
    "" using 1:6:($6*(1-$7)):($6*(1+$7)) with errorbars lt rgb "orange" linewidth 3 pointtype 7 pointsize 1 notitle,\
    "UF_d13_p0.07.txt" using 1:6 with linespoints lt rgb "skyblue" linewidth 3 pointtype 11 pointsize 1 title "UnionFind, d = 13",\
    "" using 1:6:($6*(1-$7)):($6*(1+$7)) with errorbars lt rgb "skyblue" linewidth 3 pointtype 11 pointsize 1 notitle

system("ps2pdf -dEPSCrop code_capacity_noise_model.eps code_capacity_noise_model.pdf")

# set size 1,0.75
# set output "code_capacity_noise_model_w.eps"
# replot
# system("ps2pdf -dEPSCrop code_capacity_noise_model_w.eps code_capacity_noise_model_w.pdf")

# set size 1,0.6
# set output "code_capacity_noise_model_w_w.eps"
# replot
# system("ps2pdf -dEPSCrop code_capacity_noise_model_w_w.eps code_capacity_noise_model_w_w.pdf")
