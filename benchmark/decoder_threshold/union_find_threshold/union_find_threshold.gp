set terminal postscript eps color "Arial, 28"
set xlabel "Depolarizing Error Rate (p)" font "Arial, 28"
set ylabel "Logical Error Rate (p_L)" font "Arial, 28"
set grid ytics
set size 1,1

# data generating commands:
# cargo run --release -- tool union_find_decoder_standard_planar_benchmark [3,5,7,9,11] [8.5e-2,8.6e-2,8.7e-2,8.8e-2,8.9e-2,9e-2,9.1e-2,9.2e-2,9.3e-2,9.4e-2,9.5e-2] -p0 -b1000 -m100000000 -e100000000 --only_count_logical_x --no_y_error



# finding threshold:
# cargo run --release -- tool union_find_decoder_standard_planar_benchmark [3,5,7,9] [9e-2] -p0 -b1000 -e10000 --only_count_logical_x --no_y_error
# cargo run --release -- tool union_find_decoder_standard_planar_benchmark [3,5,7,9,11,21,31] [9.5e-2,9.6e-2,9.7e-2,9.8e-2,9.9e-2,10e-2,10.1e-2,10.2e-2,10.3e-2,10.4e-2,10.5e-2] -p0 -b1000 -m1000000 -e1000000 --only_count_logical_x --no_y_error


# set logscale x
set xrange [0.095:0.105]
set xtics ("9.5%%" 0.095, "9.7%%" 0.097, "9.9%%" 0.099, "10.1%%" 0.101, "10.3%%" 0.103, "10.5%%" 0.105)
set logscale y
set ytics ("12%%" 0.12, "13%%" 0.13, "14%%" 0.14, "15%%" 0.15, "16%%" 0.16, "17%%" 0.17)
set yrange [0.12:0.17]
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder

set output "union_find_threshold.eps"

plot "d_3.txt" using 1:5 with linespoints lt rgb "red" linewidth 5 pointtype 6 pointsize 1.5 title "d = 3",\
    "d_5.txt" using 1:5 with linespoints lt rgb "blue" linewidth 5 pointtype 2 pointsize 1.5 title "d = 5",\
    "d_7.txt" using 1:5 with linespoints lt rgb "green" linewidth 5 pointtype 2 pointsize 1.5 title "d = 7",\
    "d_9.txt" using 1:5 with linespoints lt rgb "yellow" linewidth 5 pointtype 2 pointsize 1.5 title "d = 9",\
    "d_11.txt" using 1:5 with linespoints lt rgb "purple" linewidth 5 pointtype 2 pointsize 1.5 title "d = 11",\
    "d_21.txt" using 1:5 with linespoints lt rgb "black" linewidth 5 pointtype 2 pointsize 1.5 title "d = 21",\
    # "d_31.txt" using 1:5 with linespoints lt rgb "dark-green" linewidth 5 pointtype 2 pointsize 1.5 title "d = 31",\

set output '|ps2pdf -dEPSCrop union_find_threshold.eps union_find_threshold.pdf'
replot

set size 1,0.75
set output "union_find_threshold_w.eps"
replot
set output '|ps2pdf -dEPSCrop union_find_threshold_w.eps union_find_threshold_w.pdf'
replot

set size 1,0.6
set output "union_find_threshold_w_w.eps"
replot
set output '|ps2pdf -dEPSCrop union_find_threshold_w_w.eps union_find_threshold_w_w.pdf'
replot
