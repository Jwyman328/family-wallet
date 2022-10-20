#set up regtest environment in separate container
nigiri start
sleep 10 #wait for regtest nigiri to start up

docker-compose up -d # start up postgres container

cargo build
cargo watch -x 'run'  # FYI in prod mode we would want cargo run not cargo watch