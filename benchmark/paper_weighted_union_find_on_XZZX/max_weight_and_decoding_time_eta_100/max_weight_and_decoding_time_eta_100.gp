set terminal postscript eps color "Arial, 22"
set xlabel "Maximum Weight" font "Arial, 22"
set ylabel "Decoding Time (ms)" font "Arial, 22"
set grid ytics
set size 1,1.1

# set logscale x
set xrange [1:50]
# print(", ".join([f"'{i}' {i}" for i in range(1,50,3)]))
set xtics ('2' 2, '6' 6, '10' 10, '14' 14, '18' 18, '22' 22, '26' 26, '30' 30, '34' 34, '38' 38, '42' 42, '46' 46, '50' 50)
# set logscale y
# print(", ".join([f"'{i}' {i*0.001}" for i in range(0, 17, 2)]))
set ytics ('0' 0.0, '2' 0.002, '4' 0.004, '6' 0.006, '8' 0.008, '10' 0.01, '12' 0.012, '14' 0.014, '16' 0.016)
set yrange [0:1.6e-2]
set key outside horizontal top center font "Arial, 22"

# set key Left
set style fill transparent solid 0.2 noborder
set key samplen 4
set key maxrows 3
# set key height 5

set output "max_weight_and_decoding_time_eta_100.eps"

plot "data_4.txt" using 1:6 with linespoints lt rgb "red" linewidth 3 pointtype 7 pointsize 1 title "d=4 1.1x",\
    "" using 1:8 with linespoints lt rgb "blue" linewidth 3 pointtype 7 pointsize 1 title "d=4 2x",\
    "" using 1:11 with linespoints lt rgb "green" linewidth 3 pointtype 7 pointsize 1 title "d=4 Avrage",\
    "data_5.txt" using 1:6 with linespoints lt rgb "light-red" linewidth 3 pointtype 11 pointsize 1 title "d=5 1.1x",\
    "" using 1:8 with linespoints lt rgb "light-blue" linewidth 3 pointtype 11 pointsize 1 title "d=5 2x",\
    "" using 1:11 with linespoints lt rgb "light-green" linewidth 3 pointtype 11 pointsize 1 title "d=5 Average"
    # "data_6.txt" using 1:6 with linespoints lt rgb "light-red" linewidth 3 pointtype 11 pointsize 1 title "d=6 1.1x",\
    # "" using 1:8 with linespoints lt rgb "light-blue" linewidth 3 pointtype 11 pointsize 1 title "d=6 2x",\
    # "" using 1:11 with linespoints lt rgb "light-green" linewidth 3 pointtype 11 pointsize 1 title "d=6 Average"

system("ps2pdf -dEPSCrop max_weight_and_decoding_time_eta_100.eps max_weight_and_decoding_time_eta_100.pdf")

# set size 1,0.75
# set output "max_weight_and_decoding_time_eta_100_w.eps"
# replot
# system("ps2pdf -dEPSCrop max_weight_and_decoding_time_eta_100_w.eps max_weight_and_decoding_time_eta_100_w.pdf")

# set size 1,0.6
# set output "max_weight_and_decoding_time_eta_100_w_w.eps"
# replot
# system("ps2pdf -dEPSCrop max_weight_and_decoding_time_eta_100_w_w.eps max_weight_and_decoding_time_eta_100_w_w.pdf")
