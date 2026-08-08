#!/bin/bash
#SBATCH --job-name=QEC-Playground=06-25-2024=14:32:15
#SBATCH --time=13:20:00
#SBATCH --mem=16G
#SBATCH --mail-type=ALL
#SBATCH --nodes=1
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=12
#SBATCH --array=0-2351
#SBATCH --out="/gpfs/gibbs/project/lin_zhong/yw729/QEC-Playground/benchmark/paper_fusion/max_tree_size_accuracy_circuit_level/slurm_jobs/%a.jobout"
#SBATCH --error="/gpfs/gibbs/project/lin_zhong/yw729/QEC-Playground/benchmark/paper_fusion/max_tree_size_accuracy_circuit_level/slurm_jobs/%a.joberror"
#SBATCH --requeue
#SBATCH --partition=scavenge

source /gpfs/gibbs/project/lin_zhong/yw729/QEC-Playground/benchmark/paper_fusion/max_tree_size_accuracy_circuit_level/slurm_jobs/job_script.sh

