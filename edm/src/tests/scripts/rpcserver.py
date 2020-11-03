#!/usr/bin/env python3
import socket
import shlex
import copy
import subprocess
import sys

def _local_dev_container_address():
    """
    Retrieves the EdgeDB-Docker's subnet address
    """
    return [
        l for l in ([
            ip for ip in socket.gethostbyname_ex(socket.gethostname())[2]
            if not ip.startswith("127.")][:1], 
            [
                [
                    (
                        s.connect(("8.8.8.8", 53)), 
                        s.getsockname()[0],
                        s.close()
                    ) for s in [
                        socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
                    ]
                ][0][1]
        ]) if l ][0][0]

def connect(local_dev_port=18888):
    """
    Opens a socket to listen on the address
    from within Docker that we anticipate
    remote procedure calls to arrive from.
    After reading the command from the test client,
    this remote agent opens a subprocess for
    the indicated RPC call, and reads in a stream of
    the output from stdout.

    The response it generates is a UTF-8 encoded byte
    stream that terminates with an ASCII Bell escape
    sequence.
    """
    local_dev_addr = _local_dev_container_address()
    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    try:
        s.bind((local_dev_addr, local_dev_port))
    except OSError:
        lines = [
            "The service could not be started.",
            "Try changing the host, port, or run-level."
        ]
        print("Help:")
        for l in lines:
            print(l)
        import sys
        sys.exit(1)

    s.listen(1)

    while True:
        conn, addr = s.accept()
        while True:
            cmd = conn.recv(1024)
            cmd = cmd.decode('utf-8')
            cmd = shlex.split(cmd)
            if cmd == ['STOP']:
                conn.close()
                s.close()
                sys.exit(0)
            else:
                rpc = subprocess.Popen(
                    cmd,
                    stdout=subprocess.PIPE, 
                    stderr=subprocess.PIPE, 
                )
                try:
                    response, _err = b'ERROR\a', b'0'
                    with rpc as proc:
                        response = proc.communicate(timeout=60)
                    while True:
                        if rpc.poll() is not None:
                            res = response
                            res += b'\a'
                            conn.send(res)
                            break
                        else:
                            pass
                    break       
                except subprocess.TimeoutExpired:
                    rpc.kill()
                    err = _err + response
                    conn.send(err)
                    conn.close()

if __name__ == "__main__":
    connect()
