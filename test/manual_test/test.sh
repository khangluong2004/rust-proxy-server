# Check for response headers. Script only checks data.
# Can't close connection because of http/2
PROXY="127.0.0.1:80"

curl -v http://www.washington.edu/ -o washington_real.txt
curl --proxy "$PROXY" -v http://www.washington.edu/ -o washington_proxy.txt

# Test header (by injecting into request for now)
curl -H "Cache-control: max-age=1234 , whatever    ,    haha , test=\"space cache whatever\"" --proxy "$PROXY" -v http://www.washington.edu/ -o washington_proxy.txt
curl -H "Cache-control: max-age=0 , whatever    ,    haha , test=\"space \\\"cache, , \\= whatever\"" --proxy "$PROXY" -v http://www.washington.edu/ -o washington_proxy.txt

# Test for anzac - Non-utf8 bytes for body
curl -H "Cache-control: max-age=0 , whatever    ,    haha , test=\"space \\\"cache, , \\= whatever\"" --proxy "$PROXY" -v http://anzac.unimelb.edu.au/ -o anzac_proxy.txt
curl -H "Cache-control: max-age=0 , whatever    ,    haha , test=\"space \\\"cache, , \\= whatever\"" -v http://anzac.unimelb.edu.au/ -o anzac_real.txt

curl -v http://anzac.unimelb.edu.au/ -o anzac_real.txt

# Task 5
diff washington_real.txt washington_proxy.txt | head -n 10

curl -H "Cache-Control: hello=\"hello world abcdefg\",hello2=\"abc\\\"efg\",private" --proxy "$PROXY" -v http://www.washington.edu/ -o washington_proxy.txt
