# Check for response headers. Script only checks data.
# Can't close connection because of http/2
PROXY="127.0.0.1:80"

curl -v http://www.washington.edu/ -o washington_real.txt
curl --proxy "$PROXY" -v http://www.washington.edu/ -o washington_proxy.txt

# Task 5
curl -H "If-Modified-Since: Sun, 04 May 2025 12:59:21 GMT" -v http://www.washington.edu/ -o washington_real.txt
diff washington_real.txt washington_proxy.txt | head -n 10

# Test header (by injecting into request for now)
curl -H "Cache-control: max-age=1234 , whatever    ,    haha , test=\"space cache whatever\"" --proxy "$PROXY" -v http://www.washington.edu/ -o washington_proxy.txt
curl -H "Cache-control: max-age=0 , whatever    ,    haha , test=\"space cache whatever\"" --proxy "$PROXY" -v http://www.washington.edu/ -o washington_proxy.txt