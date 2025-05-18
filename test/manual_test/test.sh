# Check for response headers. Script only checks data.
# Can't close connection because of http/2
PROXY="127.0.0.1:8001"

# Remove log file
rm ./curl.txt
rm ./proxy.txt
rm ./proxyerr.txt

# Check body
urls=("http://www.washington.edu/" "http://yimg.com/" "http://icio.us/" "http://rs6.net/" "http://www.faqs.org/faqs" "http://icanhazip.com/" "http://example.com/" "http://detectportal.firefox.com/" "http://info.cern.ch/" "http://anzac.unimelb.edu.au/")
count=0
for url in "${urls[@]}"; do
    count=$((count+1))
    echo "Test $url"
    echo -e "Test $url \n" >> ./proxy.txt 

    # Start up server
    ../../htproxy -p 8001 -c >> ./proxy.txt 2>> ./proxyerr.txt &
    server=$!

    curl -v "$url" -o "real_$count.txt" &>> ./curl.txt
    curl --proxy "$PROXY" -v "$url" -o "proxy_$count.txt" &>> ./curl.txt

    echo "Diff $url"
    diff "real_$count.txt" "proxy_$count.txt" | head -n 10

    # Kill server
    echo "Kill server"
    kill -9 $server
done

# Clean up test file
rm proxy_*.txt
rm real_*.txt