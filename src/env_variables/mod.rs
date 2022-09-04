use std::env;

pub struct EnvironmentVariable {
    name: &'static str,
    value: &'static str,
}

pub fn set_env_variables(){
    let mut env_variables:Vec<EnvironmentVariable> = vec![];

    // if we are in testing environmnt set the test env variables
    if cfg!(test) {
        // do test stuff
        let test_address = EnvironmentVariable{
            name: "test_address",
            value: "bcrt1q2ltw5646zcdxcj7hvv47mklqy8la6ta83p6egw"
        };
    
        let regtest_rpc = EnvironmentVariable {
            name:"regtest_rpc",
            value:"http://localhost:3000"
        };

        let nigiri_electrum_server = EnvironmentVariable {
            name:"electrum_server",
            value:"127.0.0.1:50000"
        };
    
        env_variables.push(test_address);
        env_variables.push(regtest_rpc);
        env_variables.push(nigiri_electrum_server);
    } else {
        // we are in a dev or production environment, set appropriate env variables
        let blockstream_electrum_server = EnvironmentVariable {
            name:"electrum_server",
            value:"ssl://electrum.blockstream.info:60002"
        };
        env_variables.push(blockstream_electrum_server);
    }


    for env_variable in env_variables.iter() {
        env::set_var(env_variable.name, env_variable.value);
    }
}   

#[cfg(test)]
pub mod test {
    use super::*;
    #[test]
    fn test_set_env_variables_sets_env_variables(){
        set_env_variables();
        assert_eq!(env::var("test_address").unwrap(), "bcrt1q2ltw5646zcdxcj7hvv47mklqy8la6ta83p6egw")
    }
}
