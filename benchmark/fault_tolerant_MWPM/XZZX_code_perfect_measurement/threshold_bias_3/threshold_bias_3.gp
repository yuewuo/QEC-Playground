set terminal postscript eps color "Arial, 28"
set xlabel "Depolarizing Error Rate (p)" font "Arial, 28"
set ylabel "Logical Error Rate (p_L)" font "Arial, 28"
set grid ytics
set size 1,1

# data range:
# python -c "for i in range(4): print('%.2f' % (0.18 + (i-1.5)*0.02), end=',')"
# python -c "for i in range(7): print('%.2f' % (0.22 + (i-3)*0.01), end=',')"

# data generating commands:

# roughly test threshold
# RUST_BACKTRACE=full cargo run --release -- tool fault_tolerant_benchmark [13,17,21,25] [0,0,0,0] [0.19,0.21,0.23,0.25]-p0 -m100000000 -e10000 --use_xzzx_code --shallow_error_on_bottom --bias_eta 3

# or joint commands
# RUST_BACKTRACE=full cargo run --release -- tool fault_tolerant_benchmark [13,17,21,25] [0,0,0,0] [0.19,0.20,0.21,0.22,0.23,0.24,0.25]-p0 -m100000000 -e1000000 --use_xzzx_code --shallow_error_on_bottom --bias_eta 3

set logscale x
set xrange [0.19:0.25]
# labels
# python -c "for i in range(7): print('\'%.2f\' %.2f' % tuple([0.22 + (i-3)*0.01 for j in range(2)]), end=', ')"
set xtics ('0.19' 0.19, '0.20' 0.20, '0.21' 0.21, '0.22' 0.22, '0.23' 0.23, '0.24' 0.24, '0.25' 0.25)
# labels
# python -c "for i in range(6): print('\'%.1f\' %.1f' % tuple([0.1 + i * 0.1 for j in range(2)]), end=', ')"
set logscale y
set ytics ('0.1' 0.1, '0.2' 0.2, '0.3' 0.3, '0.4' 0.4, '0.5' 0.5, '0.6' 0.6)
set yrange [0.1:0.6]
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder

set output "threshold_bias_3.eps"

set title "XZZX code {/Symbol h} = 3 (MWPM decoder)"

plot "d_13.txt" using 1:6 with linespoints lt rgb "red" linewidth 5 pointtype 6 pointsize 1.5 title "d = 13",\
    "d_17.txt" using 1:6 with linespoints lt rgb "blue" linewidth 5 pointtype 2 pointsize 1.5 title "d = 17",\
    "d_21.txt" using 1:6 with linespoints lt rgb "green" linewidth 5 pointtype 2 pointsize 1.5 title "d = 21",\
    "d_25.txt" using 1:6 with linespoints lt rgb "yellow" linewidth 5 pointtype 2 pointsize 1.5 title "d = 25"

system("ps2pdf -dEPSCrop threshold_bias_3.eps threshold_bias_3.pdf")

set size 1,0.75
set output "threshold_bias_3_w.eps"
replot
system("ps2pdf -dEPSCrop threshold_bias_3_w.eps threshold_bias_3_w.pdf")

set size 1,0.6
set output "threshold_bias_3_w_w.eps"
replot
system("ps2pdf -dEPSCrop threshold_bias_3_w_w.eps threshold_bias_3_w_w.pdf")
