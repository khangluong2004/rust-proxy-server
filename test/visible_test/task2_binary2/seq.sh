# Replace local host with the unimelb url, and add new line to all.log
rm -rf *.txt

echo "Test task2_binary2"

echo "Start proxy"
../../htproxy -p 8080 -c >output.txt 2>err.txt &
proxy=$!
curl -X POST http://unimelb-comp30023-2025.cloud.edu.au/seed -H 'Content-Type: text/plain' --data '30023'
curl -s --proxy http://localhost:8080 http://unimelb-comp30023-2025.cloud.edu.au/gen/100000 -o curl.txt
echo "Check 1"
echo "54d86c31f221503373aa265aa2956c3cbf676b8b8827da605597577d3cde77c9 curl.txt" | sha256sum -c

curl -s --proxy http://localhost:8080 http://unimelb-comp30023-2025.cloud.edu.au/gen/10000 -o curl.txt
echo "Check 2"
echo "91c0df593918e55bc230b68b65c1d6a0b52a4ffaa2bc6a4d7869c3b5018c2885 curl.txt" | sha256sum -c


curl -X POST http://unimelb-comp30023-2025.cloud.edu.au/seed -H 'Content-Type: text/plain' --data '42'
curl -s --proxy http://localhost:8080 http://unimelb-comp30023-2025.cloud.edu.au/gen/100000 -o curl.txt
echo "Check 3"
echo "54d86c31f221503373aa265aa2956c3cbf676b8b8827da605597577d3cde77c9 curl.txt" | sha256sum -c


curl -s --proxy http://localhost:8080 http://unimelb-comp30023-2025.cloud.edu.au/gen/10000 -o curl.txt
echo "Check 4"
echo "91c0df593918e55bc230b68b65c1d6a0b52a4ffaa2bc6a4d7869c3b5018c2885 curl.txt" | sha256sum -c

echo "Check log"
diff output.txt all.log | head -n 10

echo "Kill proxy"
kill -9 $proxy