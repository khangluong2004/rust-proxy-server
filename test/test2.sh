echo "Copy exe"
cp ../htproxy ./htproxy

test_task1() {
  echo "Task1"
  echo "Launch exe at port 8001"
  ./htproxy -p 8001 >task1/output.txt 2>task1/err.txt &
  port=$!

  # do tests
  echo "test1"
  python3 task1/simple.py >task1/test1.txt &
  py=$!
  sleep .5
  curl -s -H "host: localhost" 0.0.0.0:8001/simple >/dev/null

  kill -9 $py
  sleep 0.5

  echo "test2"
  python3 task1/long.py >task1/test2.txt&
  py=$!
  sleep .5
  curl -s -H "host: localhost" 0.0.0.0:8001/long >/dev/null

  kill -9 $py
  sleep 0.5

  echo "test3"
  sudo python3 task1/bytes.py >task1/test3.txt &
  py=$!
  sleep .5
  curl -s -H "host: localhost" 0.0.0.0:8001/bytes >/dev/null

  kill -9 $py

  echo "Kill exe and server"
  kill -9 $port
}


test_task2() {
  echo "Task2"
  echo "Launch exe at port 8001"
  ./htproxy -p 8001 -c >task2/output.txt 2>task2/err.txt &
  port=$!

  rm -f task2/test1.txt
  # do tests
  echo "test1"
  python3 task2/simple.py >task2/test1.txt &
  py=$!
  sleep .5

  rm -f task2/curl.txt
  curl -s -H "host: localhost" 0.0.0.0:8001/simple >>task2/curl.txt
  curl -s -H "host: localhost" 0.0.0.0:8001/simple >>task2/curl.txt

  curl -s -H "host: localhost" 0.0.0.0:8001/1 >>task2/curl.txt
  curl -s -H "host: localhost" 0.0.0.0:8001/2 >>task2/curl.txt
  curl -s -H "host: localhost" 0.0.0.0:8001/3 >>task2/curl.txt
  curl -s -H "host: localhost" 0.0.0.0:8001/4 >>task2/curl.txt
  curl -s -H "host: localhost" 0.0.0.0:8001/5 >>task2/curl.txt
  curl -s -H "host: localhost" 0.0.0.0:8001/6 >>task2/curl.txt
  curl -s -H "host: localhost" 0.0.0.0:8001/7 >>task2/curl.txt
  curl -s -H "host: localhost" 0.0.0.0:8001/8 >>task2/curl.txt
  curl -s -H "host: localhost" 0.0.0.0:8001/9 >>task2/curl.txt
  curl -s -H "host: localhost" 0.0.0.0:8001/10 >>task2/curl.txt
  curl -s -H "host: localhost" 0.0.0.0:8001/10 >>task2/curl.txt
  curl -s -H "host: localhost" 0.0.0.0:8001/11 >>task2/curl.txt


  # test lru
  curl -s -H "host: localhost" 0.0.0.0:8001/1001 >>task2/curl.txt
  curl -s -H "host: localhost" 0.0.0.0:8001/1002 >>task2/curl.txt
  curl -s -H "host: localhost" 0.0.0.0:8001/1003 >>task2/curl.txt
  curl -s -H "host: localhost" 0.0.0.0:8001/1004 >>task2/curl.txt
  curl -s -H "host: localhost" 0.0.0.0:8001/1005 >>task2/curl.txt
  curl -s -H "host: localhost" 0.0.0.0:8001/1006 >>task2/curl.txt
  curl -s -H "host: localhost" 0.0.0.0:8001/1007 >>task2/curl.txt
  curl -s -H "host: localhost" 0.0.0.0:8001/1008 >>task2/curl.txt
  curl -s -H "host: localhost" 0.0.0.0:8001/1009 >>task2/curl.txt
  curl -s -H "host: localhost" 0.0.0.0:8001/1010 >>task2/curl.txt
  curl -s -H "host: localhost" 0.0.0.0:8001/1001 >>task2/curl.txt
  curl -s -H "host: localhost" 0.0.0.0:8001/1002 >>task2/curl.txt
  curl -s -H "host: localhost" 0.0.0.0:8001/1011 >>task2/curl.txt

  sleep 0.5
  kill -9 $py
  sleep 0.5

  echo "test2"
  python3 task2/long.py >task2/test2.txt&
  py=$!
  sleep .5
  curl -s -H "host: localhost" 0.0.0.0:8001/long >/dev/null
  curl -s -H "host: localhost" 0.0.0.0:8001/long >/dev/null
  curl -s -H "host: localhost" 0.0.0.0:8001/long >/dev/null

  kill -9 $py
  sleep 0.5

  echo "Kill exe and server"
  kill -9 $port
}

#test_task1
#test_task2
echo "Done"
