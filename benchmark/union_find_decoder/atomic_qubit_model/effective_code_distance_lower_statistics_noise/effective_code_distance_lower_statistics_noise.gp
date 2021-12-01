set terminal postscript eps color "Arial, 28"
set xlabel "Pauli Ratio (p_{Pauli} / (p_{Pauli} + p_{erasure}))" font "Arial, 28"
set ylabel "d_{effective} ({/Symbol D} ln(p_L) / {/Symbol D} ln(p))" font "Arial, 28"
set grid ytics
set size 1,1

# set logscale x
set xrange [0:1]
set xtics ("0" 0, "0.2" 0.2, "0.4" 0.4, "0.6" 0.6, "0.8" 0.8, "1.0" 1.0)
# set logscale y
set yrange [2.5:6.5]
set ytics ("3" 3, "4" 4, "5" 5, "6" 6)
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder

set output "effective_code_distance_lower_statistics_noise.eps"

# set title "Atomic Qubit 95% Erasure + 5% Pauli Circuit-Level"

plot "effective_code_distance.txt" using 1:2 with linespoints lt rgb "red" linewidth 3 pointtype 6 pointsize 1 title "d_{effective} from lowest p in the simulation",\
    "" using 1:2:($2-$3):($2+$3) with errorbars lt rgb "red" linewidth 3 pointtype 6 pointsize 1 notitle

system("ps2pdf -dEPSCrop effective_code_distance_lower_statistics_noise.eps effective_code_distance_lower_statistics_noise.pdf")

# set size 1,0.75
# set output "effective_code_distance_lower_statistics_noise_w.eps"
# replot
# system("ps2pdf -dEPSCrop effective_code_distance_lower_statistics_noise_w.eps effective_code_distance_lower_statistics_noise_w.pdf")

# set size 1,0.6
# set output "effective_code_distance_lower_statistics_noise_w_w.eps"
# replot
# system("ps2pdf -dEPSCrop effective_code_distance_lower_statistics_noise_w_w.eps effective_code_distance_lower_statistics_noise_w_w.pdf")
