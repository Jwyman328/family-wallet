#set up regtest environment in separate container
nigiri start
sleep 10 #wait for regtest nigiri to start up

docker-compose up -d # start up postgres container

cargo build
cargo run # start up web server