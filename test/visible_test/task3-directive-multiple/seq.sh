# Replace local host with the unimelb url, and add new line to all.log
rm -rf *.txt

echo "Test task2_binary2"

echo "Start proxy"
../../htproxy -p 8080 -c >output.txt 2>err.txt &
proxy=$!
curl -X POST http://unimelb-comp30023-2025.cloud.edu.au/a --data '1'
curl -s --proxy http://localhost:8080 http://unimelb-comp30023-2025.cloud.edu.au/a?cache-control=must-revalidate,PRIVate -o curl.txt
echo "Check 1"
diff "response.0" "curl.txt" | head -n 10

curl -X POST http://unimelb-comp30023-2025.cloud.edu.au/a --data '2'
curl -s --proxy http://localhost:8080 http://unimelb-comp30023-2025.cloud.edu.au/a -o curl.txt
echo "Check 2"
diff "response.1" "curl.txt" | head -n 10

curl -X POST http://unimelb-comp30023-2025.cloud.edu.au/a --data '3'
curl -s --proxy http://localhost:8080 http://unimelb-comp30023-2025.cloud.edu.au/a?cache-control=must-revalidate,PRIVate -o curl.txt
echo "Check 3"
diff "response.2" "curl.txt" | head -n 10

curl -X POST http://unimelb-comp30023-2025.cloud.edu.au/a --data '4'
curl -s --proxy http://localhost:8080 http://unimelb-comp30023-2025.cloud.edu.au/a -o curl.txt
echo "Check 4"
diff "response.1" "curl.txt" | head -n 10


echo "Check log"
diff output.txt all.log | head -n 10

echo "Kill proxy"
kill -9 $proxy