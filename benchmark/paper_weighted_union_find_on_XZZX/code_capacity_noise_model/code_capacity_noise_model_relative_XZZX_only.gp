set terminal postscript eps color "Arial, 22"
set xlabel "Noise Bias {/Symbol h} = p_Z / (p_X + p_Y)" font "Arial, 22"
set ylabel "Relative Logical Error Rate Difference" font "Arial, 22"
set grid ytics
set size 1,1

set logscale x
set xrange [0.5:50000]

# for i in range(-1, 5):
#     for j in range(1, 10):
#         number = j * (10 ** i)
#         if number < 0.5 or number > 5000:
#             continue
#         print(f"'{number if j == 5 else ''}' {number:g}", end=", ")

set xtics ('0.5' 0.5, '' 0.6, '' 0.7, '' 0.8, '' 0.9, '' 1, '' 2, '' 3, '' 4, '5' 5, '' 6, '' 7, '' 8, '' 9, '' 10, '' 20, '' 30, '' 40, '50' 50, '' 60, '' 70, '' 80, '' 90, '' 100, '' 200, '' 300, '' 400, '500' 500, '' 600, '' 700, '' 800, '' 900, '' 1000, '' 2000, '' 3000, '' 4000, '5000' 5000, '{/Symbol \245}' 50000)
# set logscale y
set ytics ("0" 0, "0.1" 0.1, "0.2" 0.2)
# set yrange [0:0.25]
set key outside horizontal top center font "Arial, 22"

set style fill transparent solid 0.2 noborder
set key samplen 2

# plot broken x axis from http://www.phyast.pitt.edu/~zov1/gnuplot/html/broken.html
A=15000;	# This is where the break point is located  
B=10000;	# This is how much is cut out of the graph 
C=0.5;		# The lower limit of the graph 
D=50000;		# The upper limit (with the cut-out) of the graph 
E1=-0.05;	# The min of the y range
E2=0.25;		# The max of the y range
eps=0.03*(E2-E1)
epsx=1500
eps2=0.03*(D-B-C)
set yrange [E1:E2]
set arrow 1 from A-eps2, E1 to A+eps2, E1 nohead lc rgb "#ffffff" front
set arrow 2 from A-eps2, E2 to A+eps2, E2 nohead lc rgb "#ffffff" front
set arrow 3 from A-epsx-eps2, E1-eps to A+epsx-eps2, E1+eps nohead front
set arrow 4 from A-epsx+eps2, E1-eps to A+epsx+eps2, E1+eps nohead front
set arrow 5 from A-epsx-eps2, E2-eps to A+epsx-eps2, E2+eps nohead front
set arrow 6 from A-epsx+eps2, E2-eps to A+epsx+eps2, E2+eps nohead front

# set title "XZZX Surface Code p = 0.07"

set output "code_capacity_noise_model_relative_XZZX_only.eps"

plot "relative_d11_p0.07.txt" using 1:2 with linespoints lt rgb "red" linewidth 3 pointtype 7 pointsize 1 title "XZZX, d = 11",\
    "" using 1:2:($2-$3):($2+$3) with errorbars lt rgb "red" linewidth 3 pointtype 7 pointsize 1 notitle,\
    "relative_d13_p0.07.txt" using 1:2 with linespoints lt rgb "blue" linewidth 3 pointtype 11 pointsize 1 title "XZZX, d = 13",\
    "" using 1:2:($2-$3):($2+$3) with errorbars lt rgb "blue" linewidth 3 pointtype 11 pointsize 1 notitle

system("ps2pdf -dEPSCrop code_capacity_noise_model_relative_XZZX_only.eps code_capacity_noise_model_relative_XZZX_only.pdf")

# set size 1,0.75
# set output "code_capacity_noise_model_relative_XZZX_only_w.eps"
# replot
# system("ps2pdf -dEPSCrop code_capacity_noise_model_relative_XZZX_only_w.eps code_capacity_noise_model_relative_XZZX_only_w.pdf")

# set size 1,0.6
# set output "code_capacity_noise_model_relative_XZZX_only_w_w.eps"
# replot
# system("ps2pdf -dEPSCrop code_capacity_noise_model_relative_XZZX_only_w_w.eps code_capacity_noise_model_relative_XZZX_only_w_w.pdf")
