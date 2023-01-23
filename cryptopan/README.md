<h1>CryptoPAN IP Address Anonymization</h1>

Anonymizes IP addresses using the CryptoPAN algorithm tightly based on the GO implementation by Yawning Angel (https://github.com/Yawning/cryptopan), which is based on the original reference implementation [paper by J. Fan, J. Xu, M. Ammar, and S. Moon. (https://ieeexplore.ieee.org/abstract/document/1181415)]

CryptoPAN is a prefix-preserving, 1-1 mapping algorithm that allows for consistent anonymization of IP addresses across datasets, provided that the same 256-bit key is used. 

IPv6 anonymization is supported, but it is not known if the code conforms to the reference implementation. 

