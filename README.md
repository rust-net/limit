# limit
流量超出限制自动关机

## install
```
# show interfaces
sudo docker run --rm -it --name="test" --net="host" imgxx/limit

# help
sudo docker run --rm -it --name="test" --net="host" imgxx/limit -h

# run
sudo docker run -itd --name="limit" --restart="always" --net="host" -v limit:/limit -v /:/host imgxx/limit -i enp3s0 -l 100G
sudo docker logs -f limit
```

## dev
```
cargo run -- -i enp3s0 -l 1G
```
docker:
```
cargo build --target x86_64-unknown-linux-musl
cargo build --target aarch64-unknown-linux-musl

# docker-compose
sudo docker-compose down && sudo docker-compose up --build

# docker
sudo docker build -t limit .
sudo docker run --rm -it --name test --net host -v limit:/limit -v /:/host limit -i enp3s0 -l 1G
sudo docker rm -f test
```
