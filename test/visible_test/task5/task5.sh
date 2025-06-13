# No cache, so no c flag
rm -rf task5*.txt

echo "Test task5"

echo "Start proxy"
../../../htproxy -p 8080 -c >task5_output.txt 2>task5_err.txt &
proxy=$!
curl --proxy http://localhost:8080 http://localhost:80 -o task5_curl.txt
sleep 2
curl --proxy http://localhost:8080 http://localhost:80 -o task5_curl.txt


echo "Kill proxy"
kill -9 $proxy


