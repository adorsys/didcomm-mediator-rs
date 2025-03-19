#!/bin/bash

# Check if the current week is divisible by 3
if (( $(date +%U) % 3 == 0 )); then
  # Your actual task here
  echo "Running the job as it's the 3rd week."
else
  echo "Skipping this week."
fi
