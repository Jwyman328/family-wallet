/// BDK used a u64 as its spending amount, which represent satoshis.
/// currently this app uses a f64 and therefore must convert values to be interoporable
/// with BDK.
/// This function will take in the float bitcoin amount and return the u64 satoshis equivalent 
pub fn convert_float_to_satoshis(amount:f64) -> u64{
    let satoshis = ((amount * 100_000.0) as u64) * 1_000;
    satoshis
}

pub fn convert_satoshis_to_float(amount:u64) -> f64{
    amount as f64
}