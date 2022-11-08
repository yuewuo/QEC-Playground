set terminal postscript eps color "Arial, 24"
set xlabel "Maximum Weight" font "Arial, 24"
set ylabel "Logical Error Rate (p_L)" font "Arial, 24"
set grid ytics
set size 1,1.1

set logscale x
set xrange [1:1100]
# print(", ".join([f"'{i}' {i}" for i in range(3,13)]))
set xtics ('1' 1, '2' 2, '4' 4, '8' 8, '16' 16, '32' 32, '64' 64, '128' 128, '256' 256, '512' 512, '1024' 1024)
set logscale y
# print(", ".join([f"'1e{i}' 1e{i}" for i in range(-4, 0)]))
set ytics ('1e-4' 1e-4, '1e-3' 1e-3, '1e-2' 1e-2, '0.1' 1e-1)
set yrange [0.0000999:0.1]
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder
set key samplen 4
# set key maxrows 2
# set key height 5

set output "max_weight_and_accuracy_2.eps"

# "" using 1:($5):($5-$6):($5+$6) with errorbars lt rgb "red" linewidth 3 pointtype 11 pointsize 1 notitle,\

plot "bias_eta_10_real.txt" using 1:6 with linespoints lt rgb "#e41a1c" linewidth 3 pointtype 7 pointsize 1.3 title "{/Symbol h} = 10",\
    "" using 1:6:($6-$6*$7):($6+$6*$7) with errorbars lt rgb "#e41a1c" linewidth 3 pointtype 7 pointsize 1.3 notitle,\
    "bias_eta_15_real.txt" using 1:6 with linespoints lt rgb "#377eb8" linewidth 3 pointtype 9 pointsize 1.3 title "{/Symbol h} = 15",\
    "" using 1:6:($6-$6*$7):($6+$6*$7) with errorbars lt rgb "#377eb8" linewidth 3 pointtype 9 pointsize 1.3 notitle,\
    "bias_eta_30_real.txt" using 1:6 with linespoints lt rgb "#4daf4a" linewidth 3 pointtype 11 pointsize 1.3 title "{/Symbol h} = 30",\
    "" using 1:6:($6-$6*$7):($6+$6*$7) with errorbars lt rgb "#4daf4a" linewidth 3 pointtype 11 pointsize 1.3 notitle,\
    "bias_eta_100_real.txt" using 1:6 with linespoints lt rgb "#ff7f00" linewidth 3 pointtype 5 pointsize 1.3 title "{/Symbol h} = 100",\
    "" using 1:6:($6-$6*$7):($6+$6*$7) with errorbars lt rgb "#ff7f00" linewidth 3 pointtype 5 pointsize 1.3 notitle,\
    "bias_eta_1000_real.txt" using 1:6 with linespoints lt rgb "#984ea3" linewidth 3 pointtype 13 pointsize 1.3 title "{/Symbol h} = 1000",\
    "" using 1:6:($6-$6*$7):($6+$6*$7) with errorbars lt rgb "#984ea3" linewidth 3 pointtype 13 pointsize 1.3 notitle,\
    "bias_eta_inf_real.txt" using 1:6 with linespoints lt rgb "black" linewidth 3 pointtype 7 pointsize 1.3 title "{/Symbol h} = +{/Symbol \245}",\
    "" using 1:6:($6-$6*$7):($6+$6*$7) with errorbars lt rgb "black" linewidth 3 pointtype 7 pointsize 1.3 notitle

system("ps2pdf -dEPSCrop max_weight_and_accuracy_2.eps max_weight_and_accuracy_2.pdf")

# set size 1,0.75
# set output "max_weight_and_accuracy_2_w.eps"
# replot
# system("ps2pdf -dEPSCrop max_weight_and_accuracy_2_w.eps max_weight_and_accuracy_2_w.pdf")

# set size 1,0.6
# set output "max_weight_and_accuracy_2_w_w.eps"
# replot
# system("ps2pdf -dEPSCrop max_weight_and_accuracy_2_w_w.eps max_weight_and_accuracy_2_w_w.pdf")
