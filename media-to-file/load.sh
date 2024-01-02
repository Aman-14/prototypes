#!sh

API_URL="http://localhost:3000/file"
REQUESTS=40

for ((i=1; i<=$REQUESTS; i++)); do
    echo "Sending request $i"
    xh post $API_URL "@video.mkv" --ignore-stdin &
    # Add sleep if needed to control the rate of requests
    # sleep 0.1  # Uncomment and adjust as needed
done

echo "Load test completed"
