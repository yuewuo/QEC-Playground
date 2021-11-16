set terminal postscript eps color "Arial, 28"
set xlabel "{/Symbol h}" font "Arial, 28"
set ylabel "Logical Error Rate (p_L)" font "Arial, 28"
set grid ytics
set size 1,1

# data generating commands:

# UF
# nohup ./rust_qecp tool union_find_decoder_standard_planar_benchmark [11] [0.08] -p3 -b1000 -m100000000 -e1000000 --max_half_weight 1 --bias_eta 0.5 > uf_eta0.5.log 2>&1 &
# nohup ./rust_qecp tool union_find_decoder_standard_planar_benchmark [11] [0.08] -p3 -b1000 -m100000000 -e1000000 --max_half_weight 1 --bias_eta 1 > uf_eta1.log 2>&1 &
# nohup ./rust_qecp tool union_find_decoder_standard_planar_benchmark [11] [0.08] -p3 -b1000 -m100000000 -e1000000 --max_half_weight 1 --bias_eta 3 > uf_eta3.log 2>&1 &
# nohup ./rust_qecp tool union_find_decoder_standard_planar_benchmark [11] [0.08] -p3 -b1000 -m100000000 -e1000000 --max_half_weight 1 --bias_eta 10 > uf_eta10.log 2>&1 &
# nohup ./rust_qecp tool union_find_decoder_standard_planar_benchmark [11] [0.08] -p3 -b1000 -m100000000 -e1000000 --max_half_weight 1 --bias_eta 30 > uf_eta30.log 2>&1 &
# nohup ./rust_qecp tool union_find_decoder_standard_planar_benchmark [11] [0.08] -p3 -b1000 -m100000000 -e1000000 --max_half_weight 1 --bias_eta 100 > uf_eta100.log 2>&1 &
# nohup ./rust_qecp tool union_find_decoder_standard_planar_benchmark [11] [0.08] -p3 -b1000 -m100000000 -e1000000 --max_half_weight 1 --bias_eta 300 > uf_eta300.log 2>&1 &
# nohup ./rust_qecp tool union_find_decoder_standard_planar_benchmark [11] [0.08] -p3 -b1000 -m100000000 -e1000000 --max_half_weight 1 --bias_eta 1000 > uf_eta1000.log 2>&1 &
# nohup ./rust_qecp tool union_find_decoder_standard_planar_benchmark [11] [0.08] -p3 -b1000 -m100000000 -e1000000 --max_half_weight 1 --bias_eta 3000 > uf_eta3000.log 2>&1 &
# nohup ./rust_qecp tool union_find_decoder_standard_planar_benchmark [11] [0.08] -p3 -b1000 -m100000000 -e1000000 --max_half_weight 1 --bias_eta 10000 > uf_eta10000.log 2>&1 &
# nohup ./rust_qecp tool union_find_decoder_standard_planar_benchmark [11] [0.08] -p3 -b1000 -m100000000 -e1000000 --max_half_weight 1 --bias_eta 1000000000000 > uf_eta1000000000000.log 2>&1 &

# MWPM
# nohup ./rust_qecp tool fault_tolerant_benchmark [11] [0] [0.08] -p3-m100000000 --shallow_error_on_bottom -e1000000 --bias_eta 0.5 > ft_eta0.5.log 2>&1 &
# nohup ./rust_qecp tool fault_tolerant_benchmark [11] [0] [0.08] -p3-m100000000 --shallow_error_on_bottom -e1000000 --bias_eta 1 > ft_eta1.log 2>&1 &
# nohup ./rust_qecp tool fault_tolerant_benchmark [11] [0] [0.08] -p3-m100000000 --shallow_error_on_bottom -e1000000 --bias_eta 3 > ft_eta3.log 2>&1 &
# nohup ./rust_qecp tool fault_tolerant_benchmark [11] [0] [0.08] -p3-m100000000 --shallow_error_on_bottom -e1000000 --bias_eta 10 > ft_eta10.log 2>&1 &
# nohup ./rust_qecp tool fault_tolerant_benchmark [11] [0] [0.08] -p3-m100000000 --shallow_error_on_bottom -e1000000 --bias_eta 30 > ft_eta30.log 2>&1 &
# nohup ./rust_qecp tool fault_tolerant_benchmark [11] [0] [0.08] -p3-m100000000 --shallow_error_on_bottom -e1000000 --bias_eta 100 > ft_eta100.log 2>&1 &
# nohup ./rust_qecp tool fault_tolerant_benchmark [11] [0] [0.08] -p3-m100000000 --shallow_error_on_bottom -e1000000 --bias_eta 300 > ft_eta300.log 2>&1 &
# nohup ./rust_qecp tool fault_tolerant_benchmark [11] [0] [0.08] -p3-m100000000 --shallow_error_on_bottom -e1000000 --bias_eta 1000 > ft_eta1000.log 2>&1 &
# nohup ./rust_qecp tool fault_tolerant_benchmark [11] [0] [0.08] -p3-m100000000 --shallow_error_on_bottom -e1000000 --bias_eta 3000 > ft_eta3000.log 2>&1 &
# nohup ./rust_qecp tool fault_tolerant_benchmark [11] [0] [0.08] -p3-m100000000 --shallow_error_on_bottom -e1000000 --bias_eta 10000 > ft_eta10000.log 2>&1 &
# nohup ./rust_qecp tool fault_tolerant_benchmark [11] [0] [0.08] -p3-m100000000 --shallow_error_on_bottom -e1000000 --bias_eta 1000000000000 > ft_eta1000000000000.log 2>&1 &

set logscale x
set xrange [0.5:100000]
set xtics ('1' 1, '1e1' 10, '1e2' 100, '1e3' 1000, '1e4' 10000, '{/Symbol \245}' 100000)
set logscale y
set ytics ("0.001" 0.001, "0.003" 0.003, "0.03" 0.03, "0.03" 0.03, "0.1" 0.1)
set yrange [0.001:0.1]
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder

set title "CSS Surface Code d = 11, p = 0.08"

set output "css_accuracy_loss_changing_with_eta_d11.eps"

plot "mwpm_d11.txt" using 1:7 with linespoints lt rgb "red" linewidth 5 pointtype 6 pointsize 1.5 title "MWPM Decoder",\
    "uf_d11.txt" using 1:6 with linespoints lt rgb "blue" linewidth 5 pointtype 2 pointsize 1.5 title "UnionFind Decoder"

system("ps2pdf -dEPSCrop css_accuracy_loss_changing_with_eta_d11.eps css_accuracy_loss_changing_with_eta_d11.pdf")

set size 1,0.75
set output "css_accuracy_loss_changing_with_eta_d11_w.eps"
replot
system("ps2pdf -dEPSCrop css_accuracy_loss_changing_with_eta_d11_w.eps css_accuracy_loss_changing_with_eta_d11_w.pdf")

set size 1,0.6
set output "css_accuracy_loss_changing_with_eta_d11_w_w.eps"
replot
system("ps2pdf -dEPSCrop css_accuracy_loss_changing_with_eta_d11_w_w.eps css_accuracy_loss_changing_with_eta_d11_w_w.pdf")
