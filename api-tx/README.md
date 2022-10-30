# bpl-api-tx

Api for serving transactions in accordance with the Solana Pay specification and for creating promo tokens.

## Development Notes
* https://medium.com/codemonday/access-wsl-localhost-from-lan-for-mobile-testing-8635697f008

Set up these port forwards for windows to wsl
```
netsh interface portproxy add v4tov4 listenport=8080 listenaddress=0.0.0.0 connectport=8080 connectaddress=172.31.54.39
```

Delete with:
```
netsh interface portproxy delete v4tov4 listenport=8080 listenaddress=0.0.0.0
```
