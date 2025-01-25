#!/bin/bash

if [ $1 == "create" ]; then 
sudo ip netns add ns1
sudo ip netns add ns2

sudo ip link add veth0 type veth peer name veth1

sudo ip link set veth0 netns ns1
sudo ip link set veth1 netns ns2

sudo ip netns exec ns1 ip addr add 10.0.0.1/24 dev veth0
sudo ip netns exec ns1 ip link set veth0 up

sudo ip netns exec ns2 ip addr add 10.0.0.2/24 dev veth1
sudo ip netns exec ns2 ip link set veth1 up
fi

if [ $1 == "delete" ]; then
sudo ip netns delete ns1
sudo ip netns delete ns2
fi
