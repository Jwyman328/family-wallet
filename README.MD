# Setting up the Testing env 
- ### follow instructions from https://github.com/vulpemventures/nigiri to download nigiri, a regtest docker implementation
- ### once nigiri is downloaded you can run 
```bash
$ bash set_up_testing_env.sh
```
- ### this will start up regtest and add 10 btc to the listed address
- ### to tear down the testing environment run 
```bash
$ bash tear_down_testing_env.sh
```

# Start up dev environment
- This will set up a regtest environment in a docker container.
- In a separate docker container a postgres database will be initiated with the default schema defined in db/sql/init
- The rust web server will be started locally, connecting to the nigiri regtest test docker container and connecting to the postgres database in a separate docker contianer.
```bash
$ bash set_up_dev_testing_env.sh
```
- ### to tear down the dev environment first stop the terminal where the rust server is running with control x control c
- then run 
```bash
$ bash tear_down_dev_env.sh
```