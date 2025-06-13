# Replace local host with the unimelb url, and add new line to all.log
rm -rf *.txt

echo "Test task2_binary2"

echo "Start proxy"
../../htproxy -p 8080 -c >output.txt 2>err.txt &
proxy=$!

# Cache, max-age=1
curl -X POST http://unimelb-comp30023-2025.cloud.edu.au/echo4 --data '(0)'
curl -X POST http://unimelb-comp30023-2025.cloud.edu.au/echo4 --data 'no-transform, comp30023="max-age=9, 2025", max-age=1'
curl -s --proxy http://localhost:8080 http://unimelb-comp30023-2025.cloud.edu.au/echo4 -H "X-COMP30023-2025: HI" -o curl.txt
echo "Check 1"
diff "response.0" "curl.txt" | head -n 10

sleep 2

# Stale, cache new
curl -X POST http://unimelb-comp30023-2025.cloud.edu.au/echo4 --data '(1)'
curl -X POST http://unimelb-comp30023-2025.cloud.edu.au/echo4 --data 'no-transform, max-age=10'
curl -s --proxy http://localhost:8080 http://unimelb-comp30023-2025.cloud.edu.au/echo4 -H "X-COMP30023-2025: HI" -o curl.txt
echo "Check 2"
diff "response.1" "curl.txt" | head -n 10

# Server cached
curl -X POST http://unimelb-comp30023-2025.cloud.edu.au/echo4 --data '(1)'
curl -s --proxy http://localhost:8080 http://unimelb-comp30023-2025.cloud.edu.au/echo4 -H 'X-COMP30023-2025: HI' -o curl.txt
echo "Check 3"
diff "response.1" "curl.txt" | head -n 10 

echo "Check log"
diff output.txt all.log | head -n 10

echo "Kill proxy"
kill -9 $proxy