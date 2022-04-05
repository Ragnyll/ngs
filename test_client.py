import socket
target_host = "127.0.0.1"
target_port = 6142
# create a socket connection
client = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
# let the client connect
client.connect((target_host, target_port))
# send some data
client.send(b'Harold\n')
response = client.recv(4096)

while True:
    msg = input('message: ')
    client.send(bytes(msg + '\n', 'utf-8'))
    print(response)
