# Replace local host with the unimelb url, and add new line to all.log
rm -rf *.txt

echo "Test task2_res_long"

echo "Start proxy"
../../htproxy -p 8080 -c >output.txt 2>err.txt &
proxy=$!

curl -X POST http://unimelb-comp30023-2025.cloud.edu.au/seed -H 'Content-Type: text/plain' --data '30023'
curl -s --proxy http://localhost:8080 http://unimelb-comp30023-2025.cloud.edu.au/gen-plain/102399 -o curl.txt
echo "Check 1"
echo "b8599f8f1ae664c437a3d29b61b215901bdb581f4b9b36e3f77a38088bac3b07 curl.txt" | sha256sum -c

curl -X POST http://unimelb-comp30023-2025.cloud.edu.au/seed -H 'Content-Type: text/plain' --data '30024'
curl -s --proxy http://localhost:8080 http://unimelb-comp30023-2025.cloud.edu.au/gen-plain/102399 -o curl.txt
echo "Check 2"
echo "bc26d5579c566cc70bca75d7c6614264def44dcdadc10eb05fc28dce09c275a7 curl.txt" | sha256sum -c


echo "Check log"
diff output.txt all.log | head -n 10

echo "Kill proxy"
kill -9 $proxy