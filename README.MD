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
$ bash set_up_dev_env.sh
```
- ### to tear down the dev environment first stop the terminal where the rust server is running with control x control c
- then run 
```bash
$ bash tear_down_dev_env.sh
```


### Logging 
Logging is currently handled by env_logger. It is initiated in the main_api with 
```rs
env_logger::init();
```
 and 
a default middleware logger is set in actix_web 
```rs
App::new().wrap(middleware::Logger::default())
```
The log level can be set with an environment variable which is currently done 
in the database_env (even though this is a bad place to put this env variable)
```rs 
RUST_LOG=info # set api level of logging 
```

To implement a specific log in a file use the log package. A bit strange since we 
are implementing the env_logger package but using the log package to emit out logs  but 
that is how it is done.
```rs
use log::{info};
info!("A new user endpoint additional log");
```