set terminal postscript eps color "Arial, 28"
set xlabel "Depolarizing Error Rate (p)" font "Arial, 28"
set ylabel "Logical Error Rate (p_L)" font "Arial, 28"
set grid ytics
set size 1,1

# data generating commands:
# cargo run --release -- tool fault_tolerant_benchmark [3,5,7,9] [0,0,0,0] [10e-2,10.1e-2,10.2e-2,10.3e-2,10.4e-2,10.5e-2,10.6e-2,10.7e-2,10.8e-2,10.9e-2,11e-2] -p0 -b1000 -e100000000 --shallow_error_on_bottom --only_count_logical_x --no_autotune --no_y_error
# cargo run --release -- tool fault_tolerant_benchmark [11] [0] [10e-2,10.1e-2,10.2e-2,10.3e-2,10.4e-2,10.5e-2,10.6e-2,10.7e-2,10.8e-2,10.9e-2,11e-2] -p0 -b100 -e10000000 -m10000000 --shallow_error_on_bottom --only_count_logical_x --no_autotune --no_y_error
# cargo run --release -- tool fault_tolerant_benchmark [21,31] [0,0] [10e-2,10.1e-2,10.2e-2,10.3e-2,10.4e-2,10.5e-2,10.6e-2,10.7e-2,10.8e-2,10.9e-2,11e-2] -p0 -b100 -e1000000 -m1000000 --shallow_error_on_bottom --only_count_logical_x --no_autotune --no_y_error





# cargo run --release -- tool fault_tolerant_benchmark [20] [0] [9e-2,9.1e-2,9.2e-2,9.3e-2,9.4e-2,9.5e-2,9.6e-2,9.7e-2,9.8e-2,9.9e-2,10e-2,10.1e-2,10.2e-2,10.3e-2,10.4e-2,10.5e-2,10.6e-2,10.7e-2,10.8e-2,10.9e-2,11e-2] -p0 -b100 -e10000000 -m10000000 --shallow_error_on_bottom --only_count_logical_x --no_autotune --no_y_error
# cargo run --release -- tool fault_tolerant_benchmark [30] [0] [9e-2,9.1e-2,9.2e-2,9.3e-2,9.4e-2,9.5e-2,9.6e-2,9.7e-2,9.8e-2,9.9e-2,10e-2,10.1e-2,10.2e-2,10.3e-2,10.4e-2,10.5e-2,10.6e-2,10.7e-2,10.8e-2,10.9e-2,11e-2] -p0 -b10 -e1000000 -m1000000 --shallow_error_on_bottom --only_count_logical_x --no_autotune --no_y_error


# finding threshold:
# cargo run --release -- tool fault_tolerant_benchmark [3,5,7] [0,0,0] [10e-2] -p0 -b1000 -e1000000 --shallow_error_on_bottom --only_count_logical_x --no_autotune --no_y_error


# set logscale x
set xrange [0.100:0.110]
set xtics ("10.0%%" 0.100, "10.2%%" 0.102, "10.4%%" 0.104, "10.6%%" 0.106, "10.8%%" 0.108, "11.0%%" 0.110)
set logscale y
set ytics ("13%%" 0.13, "14%%" 0.14, "15%%" 0.15, "16%%" 0.16, "17%%" 0.17, "18%%" 0.18)
set yrange [0.13:0.18]
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder

set output "mwpm_decoder_threshold.eps"

plot "d_3.txt" using 1:6 with linespoints lt rgb "red" linewidth 5 pointtype 6 pointsize 1.5 title "d = 3",\
    "d_5.txt" using 1:6 with linespoints lt rgb "blue" linewidth 5 pointtype 2 pointsize 1.5 title "d = 5",\
    "d_7.txt" using 1:6 with linespoints lt rgb "green" linewidth 5 pointtype 2 pointsize 1.5 title "d = 7",\
    "d_9.txt" using 1:6 with linespoints lt rgb "yellow" linewidth 5 pointtype 2 pointsize 1.5 title "d = 9",\
    "d_11.txt" using 1:6 with linespoints lt rgb "purple" linewidth 5 pointtype 2 pointsize 1.5 title "d = 11",\
    "d_21.txt" using 1:6 with linespoints lt rgb "black" linewidth 5 pointtype 2 pointsize 1.5 title "d = 21",\
    "d_31.txt" using 1:6 with linespoints lt rgb "dark-green" linewidth 5 pointtype 2 pointsize 1.5 title "d = 31",\

set output '|ps2pdf -dEPSCrop mwpm_decoder_threshold.eps mwpm_decoder_threshold.pdf'
replot

set size 1,0.75
set output "mwpm_decoder_threshold_w.eps"
replot
set output '|ps2pdf -dEPSCrop mwpm_decoder_threshold_w.eps mwpm_decoder_threshold_w.pdf'
replot

set size 1,0.6
set output "mwpm_decoder_threshold_w_w.eps"
replot
set output '|ps2pdf -dEPSCrop mwpm_decoder_threshold_w_w.eps mwpm_decoder_threshold_w_w.pdf'
replot
