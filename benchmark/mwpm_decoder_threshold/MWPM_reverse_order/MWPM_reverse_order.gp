set terminal postscript eps color "Arial, 28"
set xlabel "Depolarizing Error Rate (p)" font "Arial, 28"
set ylabel "Logical Error Rate (p_L)" font "Arial, 28"
set grid ytics
set size 1,1

# data generating commands:
# cargo run --release --features MWPM_reverse_order -- tool fault_tolerant_benchmark [3,5,7,9,11] [0,0,0,0,0] [9e-2,9.1e-2,9.2e-2,9.3e-2,9.4e-2,9.5e-2,9.6e-2,9.7e-2,9.8e-2,9.9e-2,10e-2] -p0 -b1000 -e100000000 --shallow_error_on_bottom --only_count_logical_x --no_autotune --no_y_error


# finding threshold:
# cargo run --release --features MWPM_reverse_order -- tool fault_tolerant_benchmark [3,5,7] [0,0,0] [10e-2] -p0 -b1000 -e1000000 --shallow_error_on_bottom --only_count_logical_x --no_autotune --no_y_error


set logscale x
set xrange [0.09:0.1]
set xtics 0.09,1.05,0.1
set logscale y
set ytics 0.1,1.1,0.2
set yrange [0.1:0.15]
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder

set output "MWPM_reverse_order.eps"

plot "d_3.txt" using 1:6 with linespoints lt rgb "red" linewidth 5 pointtype 6 pointsize 1.5 title "d = 3",\
    "d_5.txt" using 1:6 with linespoints lt rgb "blue" linewidth 5 pointtype 2 pointsize 1.5 title "d = 5",\
    "d_7.txt" using 1:6 with linespoints lt rgb "green" linewidth 5 pointtype 2 pointsize 1.5 title "d = 7",\
    # "d_9.txt" using 1:6 with linespoints lt rgb "yellow" linewidth 5 pointtype 2 pointsize 1.5 title "d = 9",\
    # "d_11.txt" using 1:6 with linespoints lt rgb "purple" linewidth 5 pointtype 2 pointsize 1.5 title "d = 11"

set output '|ps2pdf -dEPSCrop MWPM_reverse_order.eps MWPM_reverse_order.pdf'
replot

set size 1,0.75
set output "MWPM_reverse_order_w.eps"
replot
set output '|ps2pdf -dEPSCrop MWPM_reverse_order_w.eps MWPM_reverse_order_w.pdf'
replot

set size 1,0.6
set output "MWPM_reverse_order_w_w.eps"
replot
set output '|ps2pdf -dEPSCrop MWPM_reverse_order_w_w.eps MWPM_reverse_order_w_w.pdf'
replot
