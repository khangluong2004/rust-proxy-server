# Replace local host with the unimelb url, and add new line to all.log
rm -rf *.txt

echo "Test task2_evict_nocache"

echo "Start proxy"
../../htproxy -p 8080 -c >output.txt 2>err.txt &
proxy=$!

curl -s --proxy http://localhost:8080 http://unimelb-comp30023-2025.cloud.edu.au/echo/0 > curl.txt
echo "0"
diff curl.txt response.0 | head -n 10

curl -s --proxy http://localhost:8080 http://unimelb-comp30023-2025.cloud.edu.au/echo/1 -o curl.txt
echo "1"
diff curl.txt response.1 | head -n 10

curl -s --proxy http://localhost:8080 http://unimelb-comp30023-2025.cloud.edu.au/echo/2 -o curl.txt
echo "2"
diff curl.txt response.2 | head -n 10

curl -s --proxy http://localhost:8080 http://unimelb-comp30023-2025.cloud.edu.au/echo/3 -o curl.txt
echo "3"
diff curl.txt response.3 | head -n 10

curl -s --proxy http://localhost:8080 http://unimelb-comp30023-2025.cloud.edu.au/echo/4 -o curl.txt
echo "4"
diff curl.txt response.4 | head -n 10

curl -s --proxy http://localhost:8080 http://unimelb-comp30023-2025.cloud.edu.au/echo/5 -o curl.txt
echo "5"
diff curl.txt response.5 | head -n 10

curl -s --proxy http://localhost:8080 http://unimelb-comp30023-2025.cloud.edu.au/echo/6 -o curl.txt
echo "6"
diff curl.txt response.6 | head -n 10

curl -s --proxy http://localhost:8080 http://unimelb-comp30023-2025.cloud.edu.au/echo/7 -o curl.txt
echo "7"
diff curl.txt response.7 | head -n 10

curl -s --proxy http://localhost:8080 http://unimelb-comp30023-2025.cloud.edu.au/echo/8 -o curl.txt
echo "8"
diff curl.txt response.8 | head -n 10

curl -s --proxy http://localhost:8080 http://unimelb-comp30023-2025.cloud.edu.au/echo/9 -o curl.txt
echo "9"
diff curl.txt response.9 | head -n 10

curl -X POST http://unimelb-comp30023-2025.cloud.edu.au/a --data '120000'
curl -s --proxy http://localhost:8080 http://unimelb-comp30023-2025.cloud.edu.au/a -o curl.txt
echo "10"
echo "b7d28250f817e1e8a8584ea30736efc813858d29ea8e83d22a4c13140abd65b3 curl.txt" | sha256sum -c

curl -X POST http://unimelb-comp30023-2025.cloud.edu.au/a --data '1'
curl -s --proxy http://localhost:8080 http://unimelb-comp30023-2025.cloud.edu.au/a -o curl.txt
echo "11"
diff curl.txt response.11 | head -n 10

curl -s --proxy http://localhost:8080 http://unimelb-comp30023-2025.cloud.edu.au/echo/0 -o curl.txt
echo "12"
diff curl.txt response.0 | head -n 10

echo "Check log"
diff output.txt all.log | head -n 10

echo "Kill proxy"
kill -9 $proxy