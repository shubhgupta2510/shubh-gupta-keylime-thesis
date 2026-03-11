#!/bin/bash
# Network debugging script for Keylime containers

echo "=== Docker Network Information ==="
docker network ls
docker network inspect keylime-network

echo -e "\n=== Container IP Addresses ==="
echo "Registrar:"
docker exec -it shubh-gupta-keylime-thesis-keylime-registrar-1 ip addr
echo -e "\nVerifier:"
docker exec -it shubh-gupta-keylime-thesis-keylime-verifier-1 ip addr
echo -e "\nAgent:"
docker exec -it shubh-gupta-keylime-thesis-keylime-agent-1 ip addr

echo -e "\n=== DNS Resolution Check ==="
echo "From Agent container:"
docker exec -it shubh-gupta-keylime-thesis-keylime-agent-1 getent hosts keylime-registrar
docker exec -it shubh-gupta-keylime-thesis-keylime-agent-1 getent hosts keylime-verifier

echo -e "\n=== Connectivity Check ==="
echo "From Agent to Registrar:"
docker exec -it shubh-gupta-keylime-thesis-keylime-agent-1 ping -c 3 keylime-registrar
echo -e "\nFrom Agent to Verifier:"
docker exec -it shubh-gupta-keylime-thesis-keylime-agent-1 ping -c 3 keylime-verifier

echo -e "\n=== Port Check ==="
echo "Checking Registrar ports:"
docker exec -it shubh-gupta-keylime-thesis-keylime-agent-1 nc -zvw 3 keylime-registrar 8891
docker exec -it shubh-gupta-keylime-thesis-keylime-agent-1 nc -zvw 3 keylime-registrar 8890
echo -e "\nChecking Verifier ports:"
docker exec -it shubh-gupta-keylime-thesis-keylime-agent-1 nc -zvw 3 keylime-verifier 8892