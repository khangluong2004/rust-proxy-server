import socket

server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
server.bind(('0.0.0.0', 80))
server.listen(5)

samples = [
	"a=b,c=d,e=f,\t private ,g=d",
	"a=b,c=d,e=f,private\t,g=d",
	"!,#,$,%%#,+=##!,$=%%%,yes=',no=`,private",
	"hello=\t\"hello world abcdefg\" ,    hello2=\"abc\\\"efg\",private",
	"hello=\"abc\\=efg\",private",
	"hello=\"abc\\\\efg\",private"
	"\thello=\"abc\\!efg\",    \t   private\t"
	"hello=\"abc\\'efg\"   ,private"
	"FDSFSDg*FD12=\"abc\\'#!@G&*(@!)efg\"   ,PRiVAtE",
	"PRIVATE\t,FDSFSDg*FD12=\"abc\\'#!@G&*(@!)efg\"\t\t",
	"\t\tprivaTE\t,FDSFSDg*FD12=\"abc\\'#!@G&*(@!)efg\"  \t ",
	"PRIvvATE\t,FDSFSDg*FD12=\"abc\\'#!@G&*(@!)efg\"\t\t",
    "\t\tpriva!TE\t,FDSFSDg*FD12=\"abc\\'#!@G&*(@!)efg\"  \t ",
    "FDSFSDg*FD12=\"abc\\'#!@G&*(@!)privateefg\"  \t ",
]
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
	text = samples[i]
	i = (i + 1) % len(samples)
	sock.send(b"Cache-Control: " + bytes(text.encode('ascii')) +b"\r\n")
	sock.send(b"\r\n")
	sock.send(b"Hello World!")

	sock.close()

server.close()
