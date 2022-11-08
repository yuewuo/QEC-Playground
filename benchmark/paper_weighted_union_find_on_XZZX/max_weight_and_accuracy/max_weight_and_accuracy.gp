set terminal postscript eps color "Arial, 22"
set xlabel "Maximum Weight" font "Arial, 22"
set ylabel "Logical Error Rate" font "Arial, 22"
set grid ytics
set size 1,1.1

# set logscale x
# set xrange [1:50]
set xrange [1:30]
# print(", ".join([f"'{i}' {i}" for i in range(1,50,3)]))
set xtics ('2' 2, '6' 6, '10' 10, '14' 14, '18' 18, '22' 22, '26' 26, '30' 30, '34' 34, '38' 38, '42' 42, '46' 46, '50' 50)
set logscale y
# print(", ".join([f"'1e{i}' 1e{i}" for i in range(-4, 2)]))
set ytics ('0.01' 1e-2, '0.03' 3e-2, '0.1' 1e-1, '0.3' 0.3)
set yrange [1e-2:0.3]
set key outside horizontal top center font "Arial, 22"

set style fill transparent solid 0.2 noborder
# set key samplen 4

set output "max_weight_and_accuracy.eps"

plot "bias_eta_10.txt" using 1:6 with linespoints lt rgb "red" linewidth 3 pointtype 7 pointsize 1 title "{/Symbol h} = 10",\
    "bias_eta_100.txt" using 1:6 with linespoints lt rgb "blue" linewidth 3 pointtype 7 pointsize 1 title "{/Symbol h} = 100",\
    "bias_eta_1000.txt" using 1:6 with linespoints lt rgb "green" linewidth 3 pointtype 7 pointsize 1 title "{/Symbol h} = 1000",\
    "bias_eta_inf.txt" using 1:6 with linespoints lt rgb "purple" linewidth 3 pointtype 7 pointsize 1 title "{/Symbol h} = +{/Symbol \245}"

system("ps2pdf -dEPSCrop max_weight_and_accuracy.eps max_weight_and_accuracy.pdf")

# set size 1,0.75
# set output "max_weight_and_accuracy_w.eps"
# replot
# system("ps2pdf -dEPSCrop max_weight_and_accuracy_w.eps max_weight_and_accuracy_w.pdf")

# set size 1,0.6
# set output "max_weight_and_accuracy_w_w.eps"
# replot
# system("ps2pdf -dEPSCrop max_weight_and_accuracy_w_w.eps max_weight_and_accuracy_w_w.pdf")
