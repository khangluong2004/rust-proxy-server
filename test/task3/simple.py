import socket

server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
server.bind(('0.0.0.0', 80))
server.listen(5)

methods_base = ["private", "no-cache", "no-store", "max-age=0", "must-revalidate", "proxy-revalidate"]
methods = []
for base in methods_base:
	methods += [base] * 2

i = 0
while True:
	sock, addr = server.accept()

	# take request
	req = sock.recv(4096*4)
	print('simple-cache', req, flush=True)

	# send response
	sock.send(b"HTTP/1.1 200 OK\r\n")
	sock.send(b"Content-Length: 12\r\n")
	sock.send(b"Date: Tue, 29 Oct 2024 16:56:32 GMT\r\n")
	text = methods[i]
	i += 1
	sock.send(b"Cache-Control: " + bytes(text.encode('ascii')) +b"\r\n")
	sock.send(b"\r\n")
	sock.send(b"Hello World!")

	sock.close()

server.close()
