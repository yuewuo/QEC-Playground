set terminal postscript eps color "Arial, 28"
set xlabel "Depolarizing Error Rate (p)" font "Arial, 28"
set ylabel "Logical Error Rate (p_L)" font "Arial, 28"
set grid ytics
set size 1,1

# data range:
# python -c "for i in range(4): print('%.2f' % (0.2 + (i-1.5)*0.02), end=',')"
# python -c "for i in range(7): print('%.2f' % (0.2 + (i-3)*0.01), end=',')"

# data generating commands:

# roughly test threshold
# RUST_BACKTRACE=full cargo run --release -- tool fault_tolerant_benchmark [13,17,21,25] [0,0,0,0] [0.17,0.19,0.21,0.23] -p0-m100000000 -e10000 --use_xzzx_code --shallow_error_on_bottom --bias_eta 3 --decoder UF --max_half_weight 10

# or joint commands
# RUST_BACKTRACE=full cargo run --release -- tool fault_tolerant_benchmark [13,17,21,25] [0,0,0,0] [0.17,0.18,0.19,0.20,0.21,0.22,0.23] -p0-m20000000 -e200000 --use_xzzx_code --shallow_error_on_bottom --bias_eta 3 --decoder UF --max_half_weight 10

set logscale x
set xrange [0.17:0.23]
# labels
# python -c "for i in range(7): print('\'%.2f\' %.2f' % tuple([0.2 + (i-3)*0.01 for j in range(2)]), end=', ')"
set xtics ('0.17' 0.17, '0.18' 0.18, '0.19' 0.19, '0.20' 0.20, '0.21' 0.21, '0.22' 0.22, '0.23' 0.23)
# labels
# python -c "for i in range(6): print('\'%.1f\' %.1f' % tuple([0.1 + i * 0.1 for j in range(2)]), end=', ')"
set logscale y
set ytics ('0.1' 0.1, '0.2' 0.2, '0.3' 0.3, '0.4' 0.4, '0.5' 0.5, '0.6' 0.6)
set yrange [0.1:0.6]
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder

set output "threshold_bias_3.eps"

set title "XZZX code {/Symbol h} = 3 (UnionFind decoder)"

plot "d_13.txt" using 1:5 with linespoints lt rgb "red" linewidth 5 pointtype 6 pointsize 1.5 title "d = 13",\
    "d_17.txt" using 1:5 with linespoints lt rgb "blue" linewidth 5 pointtype 2 pointsize 1.5 title "d = 17",\
    "d_21.txt" using 1:5 with linespoints lt rgb "green" linewidth 5 pointtype 2 pointsize 1.5 title "d = 21",\
    "d_25.txt" using 1:5 with linespoints lt rgb "yellow" linewidth 5 pointtype 2 pointsize 1.5 title "d = 25"

system("ps2pdf -dEPSCrop threshold_bias_3.eps threshold_bias_3.pdf")

set size 1,0.75
set output "threshold_bias_3_w.eps"
replot
system("ps2pdf -dEPSCrop threshold_bias_3_w.eps threshold_bias_3_w.pdf")

set size 1,0.6
set output "threshold_bias_3_w_w.eps"
replot
system("ps2pdf -dEPSCrop threshold_bias_3_w_w.eps threshold_bias_3_w_w.pdf")
