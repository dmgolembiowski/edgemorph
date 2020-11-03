import socket
import subprocess

def connect(command, address="0.0.0.0", docker_proxy_port=18888):
    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM) 
    s.connect((address, docker_proxy_port))
    s.send(command)
    buff = b''
    while True:
        response = s.recv(1024)
        buff += response
        if buff[-1] == 7:
            break
        else:
            pass
    output = buff.strip(b'\a')
    output = output.decode('utf-8')
    return output

def edbpool_exec(command: str, 
        address: str = "0.0.0.0",
        docker_proxy_port: int = 18888) -> str:
    """
    This function accepts a terminal command
    to be sent to the endbpool docker
    container, and returns the output of that
    command. Note, this API does not guarantee
    data fidelity for advanced shell capabilities 
    like piping, subshells, redirection, or calls
    lasting longer than one minute.
    
    Usage:

    >>> confirm_started = edbpool_exec(
    ...     command="python3 docker_mock_scripts/phase_3/triple_pool_server.py",
    ...     address="192.168.1.155",
    ...     docker_proxy_port=8888
    ... )

    >>> daemon = edbpool_exec("edgedb-server -b --bootstrap")

    >>> pool_process_exit = edbpool_exec(
    ...     command="kill 16542",
    ...     docker_proxy_port=18888
    ... )

    In development, this function is used for
    testing purposes with the Debian edgedb-server 
    Docker image, and is to be used for remotely
    controlling the `edbpool` user's shell environment
    so that the developer can prompt the agent to 
    execute network-related and os-level test programs.

    Warning: This procedure should not be used in a production
    environment, and doing so may expose a severe
    security vulnerability when doing so. Please
    exercise caution.

    Once the result reaches the ASCII Bell (BEL) escape
    sequence, `connect` will assume that the subprocess
    does not need to read STDOUT on the Docker container
    for further detail. Please ensure that none of the test
    output contains this character.
    """
    return _connect(command, address, docker_proxy_port)
        
def _main ():
    import sys
    while True:
        try:
            command = input("(edbpool)$ ")
            command = command.encode('utf-8')
            response = _connect(command)
            print(response)
        except KeyboardInterrupt:
            try:
                command = "KILL".encode('utf-8')
                _ = _connect(command)
                sys.exit(0)
            except ConnectionRefusedError:
                sys.exit(0)

if __name__ == "__main__":
    _main()
