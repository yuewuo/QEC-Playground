#!/bin/bash
#SBATCH --job-name=QEC-Playground=06-25-2024=10:17:05
#SBATCH --time=13:20:00
#SBATCH --mem=16G
#SBATCH --mail-type=ALL
#SBATCH --nodes=1
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=12
#SBATCH --array=345,346,347,348,504,505,506,507,537,538,539,540,544,545,546,547,564,565,566,567,568,569,570,571,592,593,594,595,616,617,618,619,648,649,650,651,656,657,658,659,665,1008,1009,1010,1011,1012,1013,1014,1015,1020,1021,1022,1023,1028,1029,1030,1031,1036,1037,1038,1039,1040,1041,1042,1043,1044,1045,1046,1047,1048,1049,1050,1051,1056,1057,1058,1059,1068,1069,1070,1071,1072,1095,1150,1151,1152,1153,1163,1164,1165,1166,1180,1181,1182,1183,1462,1483,1484,1485,1486,1487,1492,1493,1494,1495,1500,1501,1502,1503,1561,1562,1621,1622,1623,1624,1626,1627,1628,1629,1633,1634,1635,1636,1658,1659,1679,1680,1690,1715,1716
#SBATCH --out="/gpfs/gibbs/project/lin_zhong/yw729/QEC-Playground/benchmark/paper_fusion/max_tree_size_accuracy/slurm_jobs/%a.jobout"
#SBATCH --error="/gpfs/gibbs/project/lin_zhong/yw729/QEC-Playground/benchmark/paper_fusion/max_tree_size_accuracy/slurm_jobs/%a.joberror"
#SBATCH --requeue
#SBATCH --partition=scavenge

source /gpfs/gibbs/project/lin_zhong/yw729/QEC-Playground/benchmark/paper_fusion/max_tree_size_accuracy/slurm_jobs/job_script.sh

