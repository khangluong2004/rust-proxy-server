import socket

server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
server.bind(('0.0.0.0', 80))
server.listen(5)

i = 0
while True:
	sock, addr = server.accept()

	# take request
	req = sock.recv(4096*4)
	print('crash', req, flush=True)

	i += 1
	if i == 1:
		sock.send(b"HTTP/1.1 200 OK\r\n")
		sock.send(b"Date: Tue, 29 Oct 2024 16:56:32 GMT\r\n")
		sock.send(b"Content-Length: 12\r\n")
		sock.send(b"\r\n")
		sock.send(b"Hello World!")
	elif i == 2:
		sock.send(b"HTTP/1.1 200 OK\r\n")
		sock.send(b"Content-Length: 12\r\n")
		sock.send(b"\r\n")
		sock.send(b"Hello World!")
	elif i == 3:
		sock.send(b"3412\r\n")
		sock.send(b"Date: Tue, 29 Oct 2024 16:56:32 GMT\r\n")
		sock.send(b"Content-Length: 12\r\n")
		sock.send(b"\r\n")
		sock.send(b"Hello World!")
	elif i == 4:
		sock.send(b"HTTP 2312 Ofds\r\n")
		sock.send(b"fasdkjhf asdf s\r\n")
		sock.send(b"Date: Tue, 29 Oct 2024 16:56:32 GMT\r\n")
		sock.send(b"Content-Length: 12\r\n")
		sock.send(b"\r\n")
		sock.send(b"Hello World!")
	else:
		# send response
		sock.send(b"HTTP/1.1 200 OK\r\n")
		sock.send(b"Date: Tue, 29 Oct 2024 16:56:32 GMT\r\n")
		sock.send(b"Content-Length: 12\r\n")
		sock.send(b"\r\n")
		sock.send(b"Hello World!")

	sock.close()

server.close()
