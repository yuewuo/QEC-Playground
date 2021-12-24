#!/bin/bash
#SBATCH --job-name=QEC-Playground=12-23-2021=19:24:07
#SBATCH --time=24:00:00
#SBATCH --mem=16G
#SBATCH --mail-type=ALL
#SBATCH --nodes=1
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=36
#SBATCH --array=0-15
#SBATCH --out="/gpfs/loomis/project/lin_zhong/yw729/QEC-Playground/benchmark/union_find_decoder/study_weight_configs/slurm_jobs/%a.jobout"
#SBATCH --error="/gpfs/loomis/project/lin_zhong/yw729/QEC-Playground/benchmark/union_find_decoder/study_weight_configs/slurm_jobs/%a.joberror"

if [ "$SLURM_ARRAY_TASK_ID" == "0" ]; then python3 /gpfs/loomis/project/lin_zhong/yw729/QEC-Playground/benchmark/union_find_decoder/study_weight_configs/different_config_thresholds.py 0 || exit 91; fi
if [ "$SLURM_ARRAY_TASK_ID" == "1" ]; then python3 /gpfs/loomis/project/lin_zhong/yw729/QEC-Playground/benchmark/union_find_decoder/study_weight_configs/different_config_thresholds.py 1 || exit 91; fi
if [ "$SLURM_ARRAY_TASK_ID" == "2" ]; then python3 /gpfs/loomis/project/lin_zhong/yw729/QEC-Playground/benchmark/union_find_decoder/study_weight_configs/different_config_thresholds.py 2 || exit 91; fi
if [ "$SLURM_ARRAY_TASK_ID" == "3" ]; then python3 /gpfs/loomis/project/lin_zhong/yw729/QEC-Playground/benchmark/union_find_decoder/study_weight_configs/different_config_thresholds.py 3 || exit 91; fi
if [ "$SLURM_ARRAY_TASK_ID" == "4" ]; then python3 /gpfs/loomis/project/lin_zhong/yw729/QEC-Playground/benchmark/union_find_decoder/study_weight_configs/different_config_thresholds.py 4 || exit 91; fi
if [ "$SLURM_ARRAY_TASK_ID" == "5" ]; then python3 /gpfs/loomis/project/lin_zhong/yw729/QEC-Playground/benchmark/union_find_decoder/study_weight_configs/different_config_thresholds.py 5 || exit 91; fi
if [ "$SLURM_ARRAY_TASK_ID" == "6" ]; then python3 /gpfs/loomis/project/lin_zhong/yw729/QEC-Playground/benchmark/union_find_decoder/study_weight_configs/different_config_thresholds.py 6 || exit 91; fi
if [ "$SLURM_ARRAY_TASK_ID" == "7" ]; then python3 /gpfs/loomis/project/lin_zhong/yw729/QEC-Playground/benchmark/union_find_decoder/study_weight_configs/different_config_thresholds.py 7 || exit 91; fi
if [ "$SLURM_ARRAY_TASK_ID" == "8" ]; then python3 /gpfs/loomis/project/lin_zhong/yw729/QEC-Playground/benchmark/union_find_decoder/study_weight_configs/different_config_thresholds.py 8 || exit 91; fi
if [ "$SLURM_ARRAY_TASK_ID" == "9" ]; then python3 /gpfs/loomis/project/lin_zhong/yw729/QEC-Playground/benchmark/union_find_decoder/study_weight_configs/different_config_thresholds.py 9 || exit 91; fi
if [ "$SLURM_ARRAY_TASK_ID" == "10" ]; then python3 /gpfs/loomis/project/lin_zhong/yw729/QEC-Playground/benchmark/union_find_decoder/study_weight_configs/different_config_thresholds.py 10 || exit 91; fi
if [ "$SLURM_ARRAY_TASK_ID" == "11" ]; then python3 /gpfs/loomis/project/lin_zhong/yw729/QEC-Playground/benchmark/union_find_decoder/study_weight_configs/different_config_thresholds.py 11 || exit 91; fi
if [ "$SLURM_ARRAY_TASK_ID" == "12" ]; then python3 /gpfs/loomis/project/lin_zhong/yw729/QEC-Playground/benchmark/union_find_decoder/study_weight_configs/different_config_thresholds.py 12 || exit 91; fi
if [ "$SLURM_ARRAY_TASK_ID" == "13" ]; then python3 /gpfs/loomis/project/lin_zhong/yw729/QEC-Playground/benchmark/union_find_decoder/study_weight_configs/different_config_thresholds.py 13 || exit 91; fi
if [ "$SLURM_ARRAY_TASK_ID" == "14" ]; then python3 /gpfs/loomis/project/lin_zhong/yw729/QEC-Playground/benchmark/union_find_decoder/study_weight_configs/different_config_thresholds.py 14 || exit 91; fi
if [ "$SLURM_ARRAY_TASK_ID" == "15" ]; then python3 /gpfs/loomis/project/lin_zhong/yw729/QEC-Playground/benchmark/union_find_decoder/study_weight_configs/different_config_thresholds.py 15 || exit 91; fi

