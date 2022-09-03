use std::env;

pub struct EnvironmentVariable {
    name: &'static str,
    value: &'static str,
}

pub fn set_env_variables(){
    let env_variables = EnvironmentVariable{
        name: "test_address",
        value: "bcrt1q2ltw5646zcdxcj7hvv47mklqy8la6ta83p6egw"
    };

    let env_variables = [env_variables];

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
