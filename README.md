# P2P File transfer
This is an experimental command line tool to send any file between any two computers on the internet without the need of them to have public static IP addresses, and without the need of any platform that either limits the file size or imposes fees on its users. <br />

The project is mainly for devices that are behind [NAT](https://en.wikipedia.org/wiki/Network_address_translation) networks like most of todays computers. But of courese it can be used between publicly accessable computers.<br /> 

I used [TCP hole punching](https://en.wikipedia.org/wiki/TCP_hole_punching#:~:text=TCP%20hole%20punching%20is%20an,TCP%20connections%20traversing%20NAT%20gateways.) technique, with the help of a public server that works as a mating point to help both peers to discover each other. Once the two peers discover each other, you can shutdown the server.

## Required
- A server/computer with a public IP address (I used a free tier AWS EC2).
- Two computers with an internet access, doesn't matter whether they are behind NAT or not.

## Building
Make sure that you have rust toolchain installed.
- For Unix systems
```
cargo build --features unix --release
```
- For Widnows 
```
cargo build --release
```
You can find the binary file `p2p-ft` in the `target/release` directory

## Running

### On the Server
```
p2p-ft server -p <PORT_NUMBER>
```
The server should be up first, listening on the provided `PORT_NUMBER` to accept incoming connections from peers. 

### On the Sender Peer
```
p2p-ft send <SERVER_PUBLIC_IP>:<PORT_NUMBER> <PATH_TO_THE_TARGET_FILE>
```

### On the Receiver Peer
```
p2p-ft rcv <SERVER_PUBLIC_IP>:<PORT_NUMBER>
```

 
Now, both peers are connected to the server and the server has handed each peer address to the other peer.
Once you see `Connected to <PEER_IP>:<PEER_PORT> successfully!` on both peers (receiver and the sender), you can now shutdown the severs. <br />
Now both peers are connected and talking directly to each others, and you will see the file sent successfully form the sender to the receiver.

> IMPORTANT: The data transferred isn't encrypted (yet)
