# limit
流量超出限制自动关机

## install
```
sudo docker run -itd --name="limit" --restart="always" --net="host" -v /:/host imgxx/limit -i enp3s0 -l 1G
```

## dev
```
cargo run -- -i enp3s0 -l 1G
```
docker:
```
cargo build --target aarch64-unknown-linux-musl

# docker-compose
sudo docker-compose down && sudo docker-compose up --build

# docker
sudo docker build -t limit .
sudo docker run --rm -it --name test --net host -v /:/host limit -i enp3s0 -l 1G
sudo docker rm -f test
```
