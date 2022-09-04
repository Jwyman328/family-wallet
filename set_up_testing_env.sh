#set up regtest environment
nigiri start
sleep 10 #wait for regtest nigiri to start up

# fund testing address
for value in {1..10}
do
    nigiri faucet bcrt1q2ltw5646zcdxcj7hvv47mklqy8la6ta83p6egw #same address as test_address in env variables.
done
sleep 5 # sleep 5 give nigiri extra start up time before cargo test can be run.