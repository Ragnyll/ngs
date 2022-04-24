#!/bin/python3
import json
import socket

target_host = "127.0.0.1"
target_port = 6142
# create a socket connection
client = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
# let the client connect
client.connect((target_host, target_port))
# send some data
connection_request = json.dumps({"user_id": "shirogane", "opponent_request": "kaguya" }) + '\n'
client.send(connection_request.encode('utf-8'))
response = client.recv(4096)

while True:
    msg = input('message: ')
    client.send(bytes(msg + '\n', 'utf-8'))
    print(response)
