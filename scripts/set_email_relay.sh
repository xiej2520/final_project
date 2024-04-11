ip6tables -I OUTPUT -p tcp -m tcp --dport 25 -j DROP
iptables -t nat -I OUTPUT -o eth0 -p tcp -m tcp --dport 25 -j DNAT --to-destination 130.245.171.151:11587
