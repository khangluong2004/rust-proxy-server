# No cache, so no c flag
rm -rf task1_gb2*.txt

echo "Test task1_gb2"

echo "Start proxy"
../../../htproxy -p 8080 >task1_gb2_output.txt 2>task1_gb2_err.txt &
proxy=$!
curl --proxy http://localhost:8080 http://localhost:80/hidden/zeros/2000 -o task1_gb2_curl.txt
echo "2e0c654b6cba3a1e816726bae0eac481eb7fd0351633768c3c18392e0f02b619 task1_gb2_curl.txt" | sha256sum -c

echo "Kill proxy"
kill -9 $proxy


