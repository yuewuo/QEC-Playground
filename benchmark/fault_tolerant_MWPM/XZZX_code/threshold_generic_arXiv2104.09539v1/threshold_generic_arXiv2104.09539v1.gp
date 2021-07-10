set terminal postscript eps color "Arial, 28"
set xlabel "Depolarizing Error Rate (p)" font "Arial, 28"
set ylabel "Logical Error Rate (p_L)" font "Arial, 28"
set grid ytics
set size 1,1

# data range:
# python -c "for i in range(13): print('%.4f' % (0.008 + (i-6)*0.0005), end=',')"

# data generating commands:
# biased CX
# RUST_BACKTRACE=full cargo run --release -- tool fault_tolerant_benchmark [4,5,6,7,8] --djs [12,15,18,21,24] [12,15,18,21,24] [0.0050,0.0055,0.0060,0.0065,0.0070,0.0075,0.0080,0.0085,0.0090,0.0095,0.0100,0.0105,0.0110] -b10 -p0 -m100000 -e100000 --use_xzzx_code --bias_eta 100 --error_model GenericBiasedWithBiasedCX
# standard CX
# RUST_BACKTRACE=full cargo run --release -- tool fault_tolerant_benchmark [4,5,6,7,8] --djs [12,15,18,21,24] [12,15,18,21,24] [0.0050,0.0055,0.0060,0.0065,0.0070,0.0075,0.0080,0.0085,0.0090,0.0095,0.0100,0.0105,0.0110] -b10 -p0 -m100000 -e100000 --use_xzzx_code --bias_eta 100 --error_model GenericBiasedWithStandardCX

set xrange [0.00001:0.1]
set xtics ("10^{-5}" 0.00001, "10^{-4}" 0.0001, "10^{-3}" 0.001, "10^{-2}" 0.01, "10^{-1}" 0.1)
set logscale y
set ytics ("10^{-8}" 0.00000001, "10^{-7}" 0.0000001, "10^{-6}" 0.000001, "10^{-5}" 0.00001, "10^{-4}" 0.0001, "10^{-3}" 0.001, "10^{-2}" 0.01, "10^{-1}" 0.1)
set yrange [0.00000001:1]
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder

set output "XZZX_code.eps"

plot "d_3_3.txt" using 1:6 with linespoints lt rgb "red" linewidth 5 pointtype 6 pointsize 1.5 title "d = 3",\
    "d_5_5.txt" using 1:6 with linespoints lt rgb "blue" linewidth 5 pointtype 2 pointsize 1.5 title "d = 5",\
    "d_7_7.txt" using 1:6 with linespoints lt rgb "green" linewidth 5 pointtype 2 pointsize 1.5 title "d = 7",\
    "d_9_9.txt" using 1:6 with linespoints lt rgb "yellow" linewidth 5 pointtype 2 pointsize 1.5 title "d = 9",\
    "d_11_11.txt" using 1:6 with linespoints lt rgb "purple" linewidth 5 pointtype 2 pointsize 1.5 title "d = 11",\
    "d_13_13.txt" using 1:6 with linespoints lt rgb "orange" linewidth 5 pointtype 2 pointsize 1.5 title "d = 13"

set output '|ps2pdf -dEPSCrop XZZX_code.eps XZZX_code.pdf'
replot

set size 1,0.75
set output "XZZX_code_w.eps"
replot
set output '|ps2pdf -dEPSCrop XZZX_code_w.eps XZZX_code_w.pdf'
replot

set size 1,0.6
set output "XZZX_code_w_w.eps"
replot
set output '|ps2pdf -dEPSCrop XZZX_code_w_w.eps XZZX_code_w_w.pdf'
replot
