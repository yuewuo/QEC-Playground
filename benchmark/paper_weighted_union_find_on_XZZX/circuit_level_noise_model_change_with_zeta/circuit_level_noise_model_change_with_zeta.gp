set terminal postscript eps color "Arial, 22"
set xlabel "Noise Bias {/Symbol z}" font "Arial, 22"
set ylabel "Logical Error Rate (p_L)" font "Arial, 22"
set grid ytics
set size 1,1

set logscale x
set xrange [1:100000]

# for i in range(-1, 5):
#     for j in range(1, 10):
#         number = j * (10 ** i)
#         if number < 0.5 or number > 5000:
#             continue
#         print(f"'{number if j == 5 else ''}' {number:g}", end=", ")

set xtics ('1' 1, '' 2, '' 3, '' 4, '' 5, '' 6, '' 7, '' 8, '' 9, '10' 10, '' 20, '' 30, '' 40, '' 50, '' 60, '' 70, '' 80, '' 90, '100' 100, '' 200, '' 300, '' 400, '' 500, '' 600, '' 700, '' 800, '' 900, '1000' 1000, '' 2000, '' 3000, '' 4000, '' 5000, '' 6000, '' 7000, '' 8000, '' 9000, '10000' 10000, '{/Symbol \245}' 100000)
set logscale y
set ytics ("1e-4" 0.0001, "1e-3" 0.001, "1e-2" 0.01, "1e-1" 0.1, "1" 1)
# set yrange [0.0001:0.04]
set key outside horizontal top center font "Arial, 22"

set style fill transparent solid 0.2 noborder
set key samplen 2

# plot broken x axis from http://www.phyast.pitt.edu/~zov1/gnuplot/html/broken.html
A=15000;	# This is where the break point is located  
B=10000;	# This is how much is cut out of the graph 
C=0.5;		# The lower limit of the graph 
D=50000;		# The upper limit (with the cut-out) of the graph 
E1=0.0001;	# The min of the y range
E2=1;		# The max of the y range
eps=0.2
epsx=1500
eps2=0.03*(D-B-C)
set yrange [E1:E2]
set arrow 1 from A-eps2, E1 to A+eps2, E1 nohead lc rgb "#ffffff" front
set arrow 2 from A-eps2, E2 to A+eps2, E2 nohead lc rgb "#ffffff" front
set arrow 3 from A-epsx-eps2, E1-eps*E1 to A+epsx-eps2, E1+eps*E1 nohead front
set arrow 4 from A-epsx+eps2, E1-eps*E1 to A+epsx+eps2, E1+eps*E1 nohead front
set arrow 5 from A-epsx-eps2, E2-eps*E2 to A+epsx-eps2, E2+eps*E2 nohead front
set arrow 6 from A-epsx+eps2, E2-eps*E2 to A+epsx+eps2, E2+eps*E2 nohead front

# set title "XZZX Surface Code (Circuit-level Noise Model) p = 0.008"

set output "circuit_level_noise_model_change_with_zeta.eps"

plot "MWPM_biased_d5_p0.008.txt" using 1:6 with linespoints lt rgb "red" linewidth 3 pointtype 7 pointsize 1 title "MWPM, 5x15x15",\
    "" using 1:6:($6*(1-$7)):($6*(1+$7)) with errorbars lt rgb "red" linewidth 3 pointtype 7 pointsize 1 notitle,\
    "UF_biased_d5_p0.008.txt" using 1:6 with linespoints lt rgb "blue" linewidth 3 pointtype 11 pointsize 1 title "UnionFind, 5x15x15",\
    "" using 1:6:($6*(1-$7)):($6*(1+$7)) with errorbars lt rgb "blue" linewidth 3 pointtype 11 pointsize 1 notitle,\
    "MWPM_biased_d7_p0.008.txt" using 1:6 with linespoints lt rgb "orange" linewidth 3 pointtype 7 pointsize 1 title "MWPM, 7x21x21",\
    "" using 1:6:($6*(1-$7)):($6*(1+$7)) with errorbars lt rgb "orange" linewidth 3 pointtype 7 pointsize 1 notitle,\
    "UF_biased_d7_p0.008.txt" using 1:6 with linespoints lt rgb "skyblue" linewidth 3 pointtype 11 pointsize 1 title "UnionFind, 7x21x21",\
    "" using 1:6:($6*(1-$7)):($6*(1+$7)) with errorbars lt rgb "skyblue" linewidth 3 pointtype 11 pointsize 1 notitle
system("ps2pdf -dEPSCrop circuit_level_noise_model_change_with_zeta.eps circuit_level_noise_model_change_with_zeta.pdf")

# set size 1,0.75
# set output "circuit_level_noise_model_change_with_zeta_w.eps"
# replot
# system("ps2pdf -dEPSCrop circuit_level_noise_model_change_with_zeta_w.eps circuit_level_noise_model_change_with_zeta_w.pdf")

# set size 1,0.6
# set output "circuit_level_noise_model_change_with_zeta_w_w.eps"
# replot
# system("ps2pdf -dEPSCrop circuit_level_noise_model_change_with_zeta_w_w.eps circuit_level_noise_model_change_with_zeta_w_w.pdf")
