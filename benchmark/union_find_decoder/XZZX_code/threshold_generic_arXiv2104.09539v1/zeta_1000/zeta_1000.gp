set terminal postscript eps color "Arial, 28"
set xlabel "p_z" font "Arial, 28"
set ylabel "Logical error rate" font "Arial, 28"
# set grid ytics
set size 1,1.1


# roughly test threshold commands:
# biased CX: 
# RUST_BACKTRACE=full cargo run --release -- tool fault_tolerant_benchmark [4,5,6] --djs [12,15,18] [12,15,18] [0.014]-p0 -m100000 -e3000 --use_xzzx_code --bias_eta 1000 --error_model GenericBiasedWithBiasedCX --no_stop_if_next_model_is_not_prepared --decoder UF --max_half_weight 10
# standard CX: 
# RUST_BACKTRACE=full cargo run --release -- tool fault_tolerant_benchmark [4,5,6] --djs [12,15,18] [12,15,18] [0.009]-p0 -m100000 -e3000 --use_xzzx_code --bias_eta 1000 --error_model GenericBiasedWithStandardCX --no_stop_if_next_model_is_not_prepared --decoder UF --max_half_weight 10


# data range:
# python -c "for i in range(5): print('%.4f' % (0.013 + (i-2)*0.0005), end=',')"
# python -c "for i in range(5): print('%.4f' % (0.009 + (i-2)*0.0005), end=',')"

# data generating commands:
# biased CX
# RUST_BACKTRACE=full cargo run --release -- tool fault_tolerant_benchmark [4,5,6] --djs [12,15,18] [12,15,18] [0.0120,0.0125,0.0130,0.0135,0.0140]-p0 -m100000 -e100000 --use_xzzx_code --bias_eta 1000 --error_model GenericBiasedWithBiasedCX --decoder UF --max_half_weight 10
# standard CX
# RUST_BACKTRACE=full cargo run --release -- tool fault_tolerant_benchmark [4,5,6] --djs [12,15,18] [12,15,18] [0.0080,0.0085,0.0090,0.0095,0.0100]-p0 -m100000 -e100000 --use_xzzx_code --bias_eta 1000 --error_model GenericBiasedWithStandardCX --decoder UF --max_half_weight 10

set logscale x
set xrange [0.0075:0.0145]
# labels
# python -c "for i in range(7): print('\'%.3f\' %.3f' % tuple([0.008 + i*0.001 for j in range(2)]), end=', ')"
set xtics ('0.008' 0.008, '0.009' 0.009, '0.010' 0.010, '0.011' 0.011, '0.012' 0.012, '0.013' 0.013, '0.014' 0.014)
set logscale y
set ytics ('0.02' 0.02, '0.03' 0.03, '0.05' 0.05, '' 0.06, '0.07' 0.07, '' 0.08, '' 0.09, "0.1" 0.1, '0.2' 0.2, '0.3' 0.3, '' 0.4, '0.5' 0.5)
set yrange [0.02:0.5]
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder

set title "Generaic Biased Noise Model (UF, {/Symbol z} = 1000)"

set output "zeta_1000.eps"

# to remove legend (because I don't know how to plot it just like Fig.7 in arXiv2104.09539v1)
set nokey

plot "biased_4.txt" using 1:6 with linespoints lt rgb "#e41a1c" linewidth 4 pointtype 7 pointsize 1.5 title "biased 4,12,12",\
    "" using 1:6:($6-$6*$8):($6+$6*$8) with errorbars lt rgb "#e41a1c" linewidth 4 pointtype 7 pointsize 1.5,\
    "biased_5.txt" using 1:6 with linespoints lt rgb "#377eb8" linewidth 4 pointtype 9 pointsize 1.5 title "biased 5,15,15",\
    "" using 1:6:($6-$6*$8):($6+$6*$8) with errorbars lt rgb "#377eb8" linewidth 4 pointtype 9 pointsize 1.5,\
    "biased_6.txt" using 1:6 with linespoints lt rgb "#4daf4a" linewidth 4 pointtype 11 pointsize 1.5 title "biased 6,18,18",\
    "" using 1:6:($6-$6*$8):($6+$6*$8) with errorbars lt rgb "#4daf4a" linewidth 4 pointtype 11 pointsize 1.5,\
    "standard_4.txt" using 1:6 with linespoints lt rgb "#e41a1c" linewidth 4 dashtype (1,1) pointtype 6 pointsize 1 title "standard 4,12,12",\
    "" using 1:6:($6-$6*$8):($6+$6*$8) with errorbars lt rgb "#e41a1c" linewidth 4 dashtype (1,1) pointtype 6 pointsize 1,\
    "standard_5.txt" using 1:6 with linespoints lt rgb "#377eb8" linewidth 4 dashtype (1,1) pointtype 8 pointsize 1 title "standard 5,15,15",\
    "" using 1:6:($6-$6*$8):($6+$6*$8) with errorbars lt rgb "#377eb8" linewidth 4 dashtype (1,1) pointtype 8 pointsize 1,\
    "standard_6.txt" using 1:6 with linespoints lt rgb "#4daf4a" linewidth 4 dashtype (1,1) pointtype 10 pointsize 1 title "standard 6,18,18",\
    "" using 1:6:($6-$6*$8):($6+$6*$8) with errorbars lt rgb "#4daf4a" linewidth 4 dashtype (1,1) pointtype 10 pointsize 1,\

system("ps2pdf -dEPSCrop zeta_1000.eps zeta_1000.pdf")

set size 1,0.75
set output "zeta_1000_w.eps"
replot
system("ps2pdf -dEPSCrop zeta_1000_w.eps zeta_1000_w.pdf")

set size 1,0.6
set output "zeta_1000_w_w.eps"
replot
system("ps2pdf -dEPSCrop zeta_1000_w_w.eps zeta_1000_w_w.pdf")
