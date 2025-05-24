# No cache, so no c flag
rm -rf task1_req*.txt

echo "Test task1_req"

echo "Start proxy"
../../htproxy -p 8080 >task1_req_output.txt 2>task1_req_err.txt &
proxy=$!
curl -X POST http://unimelb-comp30023-2025.cloud.edu.au/flip --data '0'
curl -s --proxy http://localhost:8080 http://unimelb-comp30023-2025.cloud.edu.au/flip -o task1_req_curl.txt
echo "Check 1"
diff task1_req_curl.txt response.false | head -n 10

curl -X POST http://unimelb-comp30023-2025.cloud.edu.au/flip --data '1'
curl -s --proxy http://localhost:8080 http://unimelb-comp30023-2025.cloud.edu.au/flip -o task1_req_curl.txt
echo "Check 2"
diff task1_req_curl.txt response.true | head -n 10

echo "Check log"
diff task1_req_output.txt all.log | head -n 10

echo "Kill proxy"
kill -9 $proxy