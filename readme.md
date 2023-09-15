you should first run 
```
cd mini-redis
cargo run --bin server
```

then start another session and run
```
cd httpServe
cargo run --bin http_server
```
now you can view 127.0.0.0.1:3000 on web and try
`/set /bin/:key /del`  
and so on