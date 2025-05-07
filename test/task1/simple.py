import socket

server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
server.bind(('0.0.0.0', 80))
server.listen(5)

sock, addr = server.accept()

# take request
req = sock.recv(4096*4)
print('simple', req)

# send response
sock.send(b"HTTP/1.1 200 OK\r\n")
sock.send(b"Content-Length: 12\r\n")
sock.send(b"Date: Tue, 29 Oct 2024 16:56:32 GMT\r\n")
sock.send(b"\r\n")
sock.send(b"Hello World!")

sock.close()
server.close()
