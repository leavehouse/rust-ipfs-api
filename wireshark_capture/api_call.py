import ipfsapi

ipfs = ipfsapi.connect('127.0.0.1', 5001)

res = ipfs.add("wireshark_capture/moloch.txt")
ipfs.cat(res['Hash'])
ipfs.version()
