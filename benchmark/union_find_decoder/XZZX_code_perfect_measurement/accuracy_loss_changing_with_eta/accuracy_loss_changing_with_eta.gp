set terminal postscript eps color "Arial, 28"
set xlabel "maximum half weight" font "Arial, 28"
set ylabel "Logical Error Rate (p_L)" font "Arial, 28"
set grid ytics
set size 1,1

# data generating commands:

# UF
# nohup ./rust_qecp tool union_find_decoder_standard_xzzx_benchmark [11,17] [0.1] -p3 -b1000 -m10000000 -e100000 --max_half_weight 10 --bias_eta 0.5 > uf_eta0.5.log 2>&1 &
# nohup ./rust_qecp tool union_find_decoder_standard_xzzx_benchmark [11,17] [0.1] -p3 -b1000 -m10000000 -e100000 --max_half_weight 10 --bias_eta 1 > uf_eta1.log 2>&1 &
# nohup ./rust_qecp tool union_find_decoder_standard_xzzx_benchmark [11,17] [0.1] -p3 -b1000 -m10000000 -e100000 --max_half_weight 10 --bias_eta 3 > uf_eta3.log 2>&1 &
# nohup ./rust_qecp tool union_find_decoder_standard_xzzx_benchmark [11,17] [0.1] -p3 -b1000 -m10000000 -e100000 --max_half_weight 10 --bias_eta 10 > uf_eta10.log 2>&1 &
# nohup ./rust_qecp tool union_find_decoder_standard_xzzx_benchmark [11,17] [0.1] -p3 -b1000 -m10000000 -e100000 --max_half_weight 10 --bias_eta 30 > uf_eta30.log 2>&1 &
# nohup ./rust_qecp tool union_find_decoder_standard_xzzx_benchmark [11,17] [0.1] -p3 -b1000 -m10000000 -e100000 --max_half_weight 10 --bias_eta 100 > uf_eta100.log 2>&1 &
# nohup ./rust_qecp tool union_find_decoder_standard_xzzx_benchmark [11,17] [0.1] -p3 -b1000 -m10000000 -e100000 --max_half_weight 10 --bias_eta 300 > uf_eta300.log 2>&1 &
# nohup ./rust_qecp tool union_find_decoder_standard_xzzx_benchmark [11,17] [0.1] -p3 -b1000 -m10000000 -e100000 --max_half_weight 10 --bias_eta 1000 > uf_eta1000.log 2>&1 &
# nohup ./rust_qecp tool union_find_decoder_standard_xzzx_benchmark [11,17] [0.1] -p3 -b1000 -m10000000 -e100000 --max_half_weight 10 --bias_eta 3000 > uf_eta3000.log 2>&1 &
# nohup ./rust_qecp tool union_find_decoder_standard_xzzx_benchmark [11,17] [0.1] -p3 -b1000 -m10000000 -e100000 --max_half_weight 10 --bias_eta 10000 > uf_eta10000.log 2>&1 &
# nohup ./rust_qecp tool union_find_decoder_standard_xzzx_benchmark [11,17] [0.1] -p3 -b1000 -m10000000 -e100000 --max_half_weight 20 --bias_eta +inf > uf_etainf.log 2>&1 &

# MWPM
# nohup ./rust_qecp tool fault_tolerant_benchmark [11,17] [0,0] [0.1] -p3 -b1000 -m10000000 --use_xzzx_code --shallow_error_on_bottom -e100000 --bias_eta 0.5 > ft_eta0.5.log 2>&1 &
# nohup ./rust_qecp tool fault_tolerant_benchmark [11,17] [0,0] [0.1] -p3 -b1000 -m10000000 --use_xzzx_code --shallow_error_on_bottom -e100000 --bias_eta 1 > ft_eta1.log 2>&1 &
# nohup ./rust_qecp tool fault_tolerant_benchmark [11,17] [0,0] [0.1] -p3 -b1000 -m10000000 --use_xzzx_code --shallow_error_on_bottom -e100000 --bias_eta 3 > ft_eta3.log 2>&1 &
# nohup ./rust_qecp tool fault_tolerant_benchmark [11,17] [0,0] [0.1] -p3 -b1000 -m10000000 --use_xzzx_code --shallow_error_on_bottom -e100000 --bias_eta 10 > ft_eta10.log 2>&1 &
# nohup ./rust_qecp tool fault_tolerant_benchmark [11,17] [0,0] [0.1] -p3 -b1000 -m10000000 --use_xzzx_code --shallow_error_on_bottom -e100000 --bias_eta 30 > ft_eta30.log 2>&1 &
# nohup ./rust_qecp tool fault_tolerant_benchmark [11,17] [0,0] [0.1] -p3 -b1000 -m10000000 --use_xzzx_code --shallow_error_on_bottom -e100000 --bias_eta 100 > ft_eta100.log 2>&1 &
# nohup ./rust_qecp tool fault_tolerant_benchmark [11,17] [0,0] [0.1] -p3 -b1000 -m10000000 --use_xzzx_code --shallow_error_on_bottom -e100000 --bias_eta 300 > ft_eta300.log 2>&1 &
# nohup ./rust_qecp tool fault_tolerant_benchmark [11,17] [0,0] [0.1] -p3 -b1000 -m10000000 --use_xzzx_code --shallow_error_on_bottom -e100000 --bias_eta 1000 > ft_eta1000.log 2>&1 &
# nohup ./rust_qecp tool fault_tolerant_benchmark [11,17] [0,0] [0.1] -p3 -b1000 -m10000000 --use_xzzx_code --shallow_error_on_bottom -e100000 --bias_eta 3000 > ft_eta3000.log 2>&1 &
# nohup ./rust_qecp tool fault_tolerant_benchmark [11,17] [0,0] [0.1] -p3 -b1000 -m10000000 --use_xzzx_code --shallow_error_on_bottom -e100000 --bias_eta 10000 > ft_eta10000.log 2>&1 &
# nohup ./rust_qecp tool fault_tolerant_benchmark [11,17] [0,0] [0.1] -p3 -b1000 -m10000000 --use_xzzx_code --shallow_error_on_bottom -e100000 --bias_eta +inf > ft_etainf.log 2>&1 &

# python -c "for i in range(1, 11): print('\'%d\' %d' % tuple([i for j in range(2)]), end=', ')"
set xrange [1:10]
set xtics ('1' 1, '2' 2, '3' 3, '4' 4, '5' 5, '6' 6, '7' 7, '8' 8, '9' 9, '10' 10)
set logscale y
set ytics ("10^{-5}" 0.00001, "10^{-4}" 0.0001, "10^{-3}" 0.001, "10^{-2}" 0.01, "10^{-1}" 0.1)
set yrange [0.00001:0.1]
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder

set title "UnionFind Decoder {/Symbol h} = 1e5"

set output "accuracy_loss_changing_with_eta.eps"

plot "p_0.1.txt" using 1:6 with linespoints lt rgb "red" linewidth 5 pointtype 6 pointsize 1.5 title "p = 0.1",\
    "p_0.05.txt" using 1:6 with linespoints lt rgb "blue" linewidth 5 pointtype 2 pointsize 1.5 title "p = 0.05",\
    "p_0.02.txt" using 1:6 with linespoints lt rgb "green" linewidth 5 pointtype 6 pointsize 1.5 title "p = 0.02"

set output '|ps2pdf -dEPSCrop accuracy_loss_changing_with_eta.eps accuracy_loss_changing_with_eta.pdf'
replot

set size 1,0.75
set output "accuracy_loss_changing_with_eta_w.eps"
replot
set output '|ps2pdf -dEPSCrop accuracy_loss_changing_with_eta_w.eps accuracy_loss_changing_with_eta_w.pdf'
replot

set size 1,0.6
set output "accuracy_loss_changing_with_eta_w_w.eps"
replot
set output '|ps2pdf -dEPSCrop accuracy_loss_changing_with_eta_w_w.eps accuracy_loss_changing_with_eta_w_w.pdf'
replot
