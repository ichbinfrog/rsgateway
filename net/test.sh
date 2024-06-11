cargo b
sudo setcap cap_net_admin=eip ../target/debug/net

../target/debug/net &
pid=$!

__clean() {
    kill -9 $pid
}

trap __clean SIGINT

sudo ip addr add 192.168.0.1/24 dev tun0
sudo ip link set up dev tun0
wait $pid
