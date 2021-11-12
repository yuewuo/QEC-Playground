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

# plot broken x axis from http://www.phyast.pitt.edu/~zov1/gnuplot/html/broken.html
A=15000;	# This is where the break point is located  
B=10000;	# This is how much is cut out of the graph 
C=0.5;		# The lower limit of the graph 
D=50000;		# The upper limit (with the cut-out) of the graph 
E1=0.0001;	# The min of the y range
E2=0.04;		# The max of the y range
eps=0.2
epsx=1500
eps2=0.01*(D-B-C)
set yrange [E1:E2]
set arrow 1 from A-eps2, E1 to A+eps2, E1 nohead lc rgb "#ffffff" front
set arrow 2 from A-eps2, E2 to A+eps2, E2 nohead lc rgb "#ffffff" front
set arrow 3 from A-epsx-eps2, E1-eps*E1 to A+epsx-eps2, E1+eps*E1 nohead front
set arrow 4 from A-epsx+eps2, E1-eps*E1 to A+epsx+eps2, E1+eps*E1 nohead front
set arrow 5 from A-epsx-eps2, E2-eps*E2 to A+epsx-eps2, E2+eps*E2 nohead front
set arrow 6 from A-epsx+eps2, E2-eps*E2 to A+epsx+eps2, E2+eps*E2 nohead front

# set title "XZZX Surface Code p = 0.07"

set output "code_capacity_noise_model_CSS.eps"


plot "MWPM_d11_p0.07.txt" using 1:6 with linespoints lt rgb "red" linewidth 3 pointtype 7 pointsize 1 title "MWPM, d = 11",\
    "" using 1:6:($6*(1-$7)):($6*(1+$7)) with errorbars lt rgb "red" linewidth 3 pointtype 7 pointsize 1 notitle,\
    "UF_d11_p0.07.txt" using 1:6 with linespoints lt rgb "blue" linewidth 3 pointtype 11 pointsize 1 title "UnionFind, d = 11",\
    "" using 1:6:($6*(1-$7)):($6*(1+$7)) with errorbars lt rgb "blue" linewidth 3 pointtype 11 pointsize 1 notitle,\
    "MWPM_d13_p0.07.txt" using 1:6 with linespoints lt rgb "orange" linewidth 3 pointtype 7 pointsize 1 title "MWPM, d = 13",\
    "" using 1:6:($6*(1-$7)):($6*(1+$7)) with errorbars lt rgb "orange" linewidth 3 pointtype 7 pointsize 1 notitle,\
    "UF_d13_p0.07.txt" using 1:6 with linespoints lt rgb "skyblue" linewidth 3 pointtype 11 pointsize 1 title "UnionFind, d = 13",\
    "" using 1:6:($6*(1-$7)):($6*(1+$7)) with errorbars lt rgb "skyblue" linewidth 3 pointtype 11 pointsize 1 notitle

system("ps2pdf -dEPSCrop code_capacity_noise_model_CSS.eps code_capacity_noise_model_CSS.pdf")

# set size 1,0.75
# set output "code_capacity_noise_model_CSS_w.eps"
# replot
# system("ps2pdf -dEPSCrop code_capacity_noise_model_CSS_w.eps code_capacity_noise_model_CSS_w.pdf")

# set size 1,0.6
# set output "code_capacity_noise_model_CSS_w_w.eps"
# replot
# system("ps2pdf -dEPSCrop code_capacity_noise_model_CSS_w_w.eps code_capacity_noise_model_CSS_w_w.pdf")
