import ipfsapi

ipfs = ipfsapi.connect('127.0.0.1', 5001)

res = ipfs.add(["wireshark_capture/moloch.txt", "LICENSE-0BSD"])
print(res)
print()
ipfs.cat(res[1]['Hash'])
print(ipfs.version())
