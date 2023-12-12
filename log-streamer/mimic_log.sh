#!/bin/bash

log_file="out.log"

# Function to log a message
log_message() {
    local timestamp
    timestamp=$(date +"%Y-%m-%d %H:%M:%S")
    echo "[$timestamp] $1" >> "$log_file"
}

# Main program
echo "Logging to $log_file"

for i in {1..50}; do
    log_message "This is log message $i"
    sleep 1
done

echo "Logging complete."
