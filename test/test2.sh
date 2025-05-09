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
  # simple response
  curl -s -H "host: localhost" 0.0.0.0:8001/simple >/dev/null

  kill -9 $py
  sleep 0.5

  echo "test2"
  python3 task1/long.py >task1/test2.txt&
  py=$!
  sleep .5
  # very long response
  curl -s -H "host: localhost" 0.0.0.0:8001/long >/dev/null

  kill -9 $py
  sleep 0.5

  echo "test3"
  sudo python3 task1/bytes.py >task1/test3.txt &
  py=$!
  sleep .5
  # non-ascii response
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
  # simple cachable response
  curl -s -H "host: localhost" 0.0.0.0:8001/simple >>task2/curl.txt
  curl -s -H "host: localhost" 0.0.0.0:8001/simple >>task2/curl.txt

  # testing LRU
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
  # should evict 1
  curl -s -H "host: localhost" 0.0.0.0:8001/11 >>task2/curl.txt


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
  # access 1 and 2
  curl -s -H "host: localhost" 0.0.0.0:8001/1001 >>task2/curl.txt
  curl -s -H "host: localhost" 0.0.0.0:8001/1002 >>task2/curl.txt
  # should evict 1003
  curl -s -H "host: localhost" 0.0.0.0:8001/1011 >>task2/curl.txt

  sleep 0.5
  kill -9 $py
  sleep 0.5

  echo "test2"
  python3 task2/long.py >task2/test2.txt&
  py=$!
  sleep .5
  # don't cache long requests
  curl -s -H "host: localhost" 0.0.0.0:8001/long >/dev/null
  curl -s -H "host: localhost" 0.0.0.0:8001/long >/dev/null
  curl -s -H "host: localhost" 0.0.0.0:8001/long >/dev/null

  kill -9 $py
  sleep 0.5

  echo "Kill exe and server"
  kill -9 $port
}


test_task3() {
  echo "Task3"
  echo "Launch exe at port 8001"
  ./htproxy -p 8001 -c >task3/output.txt 2>task3/err.txt &
  port=$!

  rm -f task3/test1.txt
  # do tests
  echo "test1"
  python3 task3/simple.py >task3/test1.txt &
  py=$!
  sleep .5

  rm -f task3/curl.txt
  # all of which should not be cached
  curl -vs -H "host: localhost" 0.0.0.0:8001/a 2>&1 | less >>task3/curl.txt
  curl -vs -H "host: localhost" 0.0.0.0:8001/a 2>&1 | less >>task3/curl.txt
  curl -vs -H "host: localhost" 0.0.0.0:8001/b 2>&1 | less >>task3/curl.txt
  curl -vs -H "host: localhost" 0.0.0.0:8001/b 2>&1 | less >>task3/curl.txt
  curl -vs -H "host: localhost" 0.0.0.0:8001/c 2>&1 | less >>task3/curl.txt
  curl -vs -H "host: localhost" 0.0.0.0:8001/c 2>&1 | less >>task3/curl.txt
  curl -vs -H "host: localhost" 0.0.0.0:8001/d 2>&1 | less >>task3/curl.txt
  curl -vs -H "host: localhost" 0.0.0.0:8001/d 2>&1 | less >>task3/curl.txt
  curl -vs -H "host: localhost" 0.0.0.0:8001/e 2>&1 | less >>task3/curl.txt
  curl -vs -H "host: localhost" 0.0.0.0:8001/e 2>&1 | less >>task3/curl.txt
  curl -vs -H "host: localhost" 0.0.0.0:8001/f 2>&1 | less >>task3/curl.txt
  curl -vs -H "host: localhost" 0.0.0.0:8001/f 2>&1 | less >>task3/curl.txt

  kill -9 $py
  sleep 0.5

  echo "test2"
  python3 task3/long.py >task3/test2.txt &
  py=$!
  sleep .5

  # complex queries that should not be cached
  curl -vs -H "host: localhost" 0.0.0.0:8001/1 2>&1 | less >>task3/curl.txt
  curl -vs -H "host: localhost" 0.0.0.0:8001/1 2>&1 | less >>task3/curl.txt
  curl -vs -H "host: localhost" 0.0.0.0:8001/2 2>&1 | less >>task3/curl.txt
  curl -vs -H "host: localhost" 0.0.0.0:8001/2 2>&1 | less >>task3/curl.txt
  curl -vs -H "host: localhost" 0.0.0.0:8001/3 2>&1 | less >>task3/curl.txt
  curl -vs -H "host: localhost" 0.0.0.0:8001/3 2>&1 | less >>task3/curl.txt
  curl -vs -H "host: localhost" 0.0.0.0:8001/4 2>&1 | less >>task3/curl.txt
  curl -vs -H "host: localhost" 0.0.0.0:8001/4 2>&1 | less >>task3/curl.txt
  curl -vs -H "host: localhost" 0.0.0.0:8001/5 2>&1 | less >>task3/curl.txt
  curl -vs -H "host: localhost" 0.0.0.0:8001/5 2>&1 | less >>task3/curl.txt
  curl -vs -H "host: localhost" 0.0.0.0:8001/6 2>&1 | less >>task3/curl.txt
  curl -vs -H "host: localhost" 0.0.0.0:8001/6 2>&1 | less >>task3/curl.txt
  curl -vs -H "host: localhost" 0.0.0.0:8001/7 2>&1 | less >>task3/curl.txt
  curl -vs -H "host: localhost" 0.0.0.0:8001/7 2>&1 | less >>task3/curl.txt
  curl -vs -H "host: localhost" 0.0.0.0:8001/8 2>&1 | less >>task3/curl.txt
  curl -vs -H "host: localhost" 0.0.0.0:8001/8 2>&1 | less >>task3/curl.txt
  curl -vs -H "host: localhost" 0.0.0.0:8001/9 2>&1 | less >>task3/curl.txt
  curl -vs -H "host: localhost" 0.0.0.0:8001/9 2>&1 | less >>task3/curl.txt
  curl -vs -H "host: localhost" 0.0.0.0:8001/10 2>&1 | less >>task3/curl.txt
  curl -vs -H "host: localhost" 0.0.0.0:8001/10 2>&1 | less >>task3/curl.txt
  curl -vs -H "host: localhost" 0.0.0.0:8001/11 2>&1 | less >>task3/curl.txt
  curl -vs -H "host: localhost" 0.0.0.0:8001/11 2>&1 | less >>task3/curl.txt

  # complex queries that should be cached
  curl -vs -H "host: localhost" 0.0.0.0:8001/1001 2>&1 | less >>task3/curl.txt
  curl -vs -H "host: localhost" 0.0.0.0:8001/1001 2>&1 | less >>task3/curl.txt
  curl -vs -H "host: localhost" 0.0.0.0:8001/1002 2>&1 | less >>task3/curl.txt
  curl -vs -H "host: localhost" 0.0.0.0:8001/1002 2>&1 | less >>task3/curl.txt
  curl -vs -H "host: localhost" 0.0.0.0:8001/1003 2>&1 | less >>task3/curl.txt
  curl -vs -H "host: localhost" 0.0.0.0:8001/1003 2>&1 | less >>task3/curl.txt

  kill -9 $py
  sleep 0.5



  echo "Kill exe and server"
  kill -9 $port
}


test_task4() {
  echo "Task4"
  echo "Launch exe at port 8001"
  ./htproxy -p 8001 -c >task4/output.txt 2>task4/err.txt &
  port=$!

  rm -f task4/test1.txt
  # do tests
  echo "test1"
  python3 task4/simple.py >task4/test1.txt &
  py=$!
  sleep .5

  rm -f task4/curl.txt
  # should be non-stale
  curl -vs -H "host: localhost" 0.0.0.0:8001/stale_test 2>&1 | less >>task4/curl.txt
  # should be non-stale
  curl -vs -H "host: localhost" 0.0.0.0:8001/stale_test 2>&1 | less >>task4/curl.txt
  sleep 4
  # should be non-stale
  curl -vs -H "host: localhost" 0.0.0.0:8001/stale_test 2>&1 | less >>task4/curl.txt
  sleep 2
  # should be stale, refetch
  curl -vs -H "host: localhost" 0.0.0.0:8001/stale_test 2>&1 | less >>task4/curl.txt
  # should be non-stale
  curl -vs -H "host: localhost" 0.0.0.0:8001/stale_test 2>&1 | less >>task4/curl.txt

  kill -9 $py
  sleep 0.5

  echo "test2"
  python3 task4/long.py >task4/test2.txt &
  py=$!
  sleep .5

  # no cache
  curl -vs -H "host: localhost" 0.0.0.0:8001/no_cache_test 2>&1 | less >>task4/curl.txt
  # should be no cache hit
  curl -vs -H "host: localhost" 0.0.0.0:8001/no_cache_test 2>&1 | less >>task4/curl.txt

  # cached with age=5
  curl -vs -H "host: localhost" 0.0.0.0:8001/stale_no_cache_test 2>&1 | less >>task4/curl.txt
  # cached
  curl -vs -H "host: localhost" 0.0.0.0:8001/stale_no_cache_test 2>&1 | less >>task4/curl.txt
  sleep 6

  # evict stale, also no cache hit
  curl -vs -H "host: localhost" 0.0.0.0:8001/stale_no_cache_test 2>&1 | less >>task4/curl.txt

  # no cache hit, but caching
  curl -vs -H "host: localhost" 0.0.0.0:8001/stale_no_cache_test 2>&1 | less >>task4/curl.txt
  # cache hit
  curl -vs -H "host: localhost" 0.0.0.0:8001/stale_no_cache_test 2>&1 | less >>task4/curl.txt

  kill -9 $py
  sleep 0.5


  echo "Kill exe and server"
  kill -9 $port
}

mkdir -p task1
mkdir -p task2
mkdir -p task3
mkdir -p task4

#test_task1
#test_task2
#test_task3
test_task4
echo "Done"
