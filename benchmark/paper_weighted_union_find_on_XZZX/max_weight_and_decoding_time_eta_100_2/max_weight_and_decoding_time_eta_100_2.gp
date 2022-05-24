set terminal postscript eps color "Arial, 24"
set xlabel "Maximum Weight" font "Arial, 24"
set ylabel "Average Decoding Time Per Measurement (s)" font "Arial, 24"
set grid ytics
set size 1,1.1

set logscale x
set xrange [1:1100]
# print(", ".join([f"'{i}' {i}" for i in range(3,13)]))
set xtics ('1' 1, '2' 2, '4' 4, '8' 8, '16' 16, '32' 32, '64' 64, '128' 128, '256' 256, '512' 512, '1024' 1024)
set logscale y
# print(", ".join([f"'1e{i}' 1e{i}" for i in range(-6, -1)]))
set ytics ('1e-6' 1e-6, '1e-5' 1e-5, '1e-4' 1e-4, '1e-3' 1e-3, '1e-2' 1e-2)
set yrange [0.000000999:0.01]
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder
set key samplen 4
# set key maxrows 2
# set key height 5

set output "max_weight_and_decoding_time_eta_100_2.eps"

intercept10 = "`tail -n2 processed_UF_10.txt | awk '{print $6}' | head -n1`"+0
intercept15 = "`tail -n2 processed_UF_15.txt | awk '{print $6}' | head -n1`"+0
intercept30 = "`tail -n2 processed_UF_30.txt | awk '{print $6}' | head -n1`"+0
intercept100 = "`tail -n2 processed_UF_100.txt | awk '{print $6}' | head -n1`"+0
intercept1000 = "`tail -n2 processed_UF_1000.txt | awk '{print $6}' | head -n1`"+0
interceptinf = "`tail -n2 processed_UF_inf.txt | awk '{print $6}' | head -n1`"+0

# "" using 1:($5):($5-$6):($5+$6) with errorbars lt rgb "red" linewidth 3 pointtype 11 pointsize 1 notitle,\

plot exp(0 * log(x) + intercept10)/150 notitle with lines dashtype 2 lt rgb "#e41a1c" linewidth 3,\
    exp(0 * log(x) + intercept15)/150 notitle with lines dashtype 2 lt rgb "#377eb8" linewidth 3,\
    exp(0 * log(x) + intercept30)/150 notitle with lines dashtype 2 lt rgb "#4daf4a" linewidth 3,\
    exp(0 * log(x) + intercept100)/150 notitle with lines dashtype 2 lt rgb "#ff7f00" linewidth 3,\
    exp(0 * log(x) + intercept1000)/150 notitle with lines dashtype 2 lt rgb "#984ea3" linewidth 3,\
    exp(0 * log(x) + interceptinf)/150 notitle with lines dashtype 2 lt rgb "black" linewidth 3,\
    "processed_UF_10.txt" using 3:($5/150) with linespoints lt rgb "#e41a1c" linewidth 3 pointtype 7 pointsize 1.3 title "{/Symbol h} = 10",\
    "processed_UF_15.txt" using 3:($5/150) with linespoints lt rgb "#377eb8" linewidth 3 pointtype 9 pointsize 1.3 title "{/Symbol h} = 15",\
    "processed_UF_30.txt" using 3:($5/150) with linespoints lt rgb "#4daf4a" linewidth 3 pointtype 11 pointsize 1.3 title "{/Symbol h} = 30",\
    "processed_UF_100.txt" using 3:($5/150) with linespoints lt rgb "#ff7f00" linewidth 3 pointtype 5 pointsize 1.3 title "{/Symbol h} = 100",\
    "processed_UF_1000.txt" using 3:($5/150) with linespoints lt rgb "#984ea3" linewidth 3 pointtype 13 pointsize 1.3 title "{/Symbol h} = 1000",\
    "processed_UF_inf.txt" using 3:($5/150) with linespoints lt rgb "black" linewidth 3 pointtype 7 pointsize 1.3 title "{/Symbol h} = +{/Symbol \245}"

system("ps2pdf -dEPSCrop max_weight_and_decoding_time_eta_100_2.eps max_weight_and_decoding_time_eta_100_2.pdf")

# set size 1,0.75
# set output "max_weight_and_decoding_time_eta_100_2_w.eps"
# replot
# system("ps2pdf -dEPSCrop max_weight_and_decoding_time_eta_100_2_w.eps max_weight_and_decoding_time_eta_100_2_w.pdf")

# set size 1,0.6
# set output "max_weight_and_decoding_time_eta_100_2_w_w.eps"
# replot
# system("ps2pdf -dEPSCrop max_weight_and_decoding_time_eta_100_2_w_w.eps max_weight_and_decoding_time_eta_100_2_w_w.pdf")
