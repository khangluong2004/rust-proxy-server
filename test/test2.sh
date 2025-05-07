echo "Copy exe"
cp ../htproxy ./htproxy

test_task1() {
  echo "Task1"
  echo "Launch exe at port 8001"
  ./htproxy -p 8001 >output.txt 2>err.txt &
  port=$!

  # do tests
  echo "test1"
  python3 task1/simple.py &
  py=$!
  sleep .5
  curl -s -H "host: localhost" 0.0.0.0:8001/simple >/dev/null

  kill -9 $py
  sleep 1

  echo "test2"
  python3 task1/long.py &
  py=$!
  sleep .5
  curl -s -H "host: localhost" 0.0.0.0:8001/long >/dev/null

  kill -9 $py
  sleep 1

  echo "test3"
  sudo python3 task1/bytes.py &
  py=$!
  sleep .5
  curl -s -H "host: localhost" 0.0.0.0:8001/bytes >/dev/null

  kill -9 $py

  echo "Kill exe and server"
  kill -9 $port
}

test_task1
echo "Done"
