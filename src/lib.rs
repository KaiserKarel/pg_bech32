use pgrx::prelude::*;

pgrx::pg_module_magic!();

extension_sql!(
    "\
CREATE TYPE Bech32 AS (
    hrp text,
    data bytea
);",
    name = "create_bech32_type",
);

const BECH_COMPOSITE_TYPE: &str = "Bech32";

/// Decode a string with a bech32 or bech32m checksum into the `Hrp` and `data` components.
#[pg_extern(immutable, parallel_safe)]
pub fn bech32_decode(input: &str) -> pgrx::composite_type!('static, BECH_COMPOSITE_TYPE) {
    let (hrp, data) = bech32::decode(input).expect("error decoding bech32");
    let mut bech = PgHeapTuple::new_composite_type(BECH_COMPOSITE_TYPE)
        .unwrap_or_else(|_| panic!("error creating {} composite type", BECH_COMPOSITE_TYPE));
    bech.set_by_name("hrp", hrp.as_str())
        .expect("error setting hrp");
    bech.set_by_name("data", data).expect("error setting data");
    bech
}

/// Encode the `Hrp` (Human Readable Part) and input into a checksummed bech32 encoded string.
/// Supports 3 modes:
/// - bech32
/// - bech32m
/// - nochecksum
#[pg_extern(immutable, parallel_safe)]
pub fn bech32_encode(hrp: &str, input: &[u8], mode: &str) -> String {
    use bech32::{Bech32, Bech32m, Hrp, NoChecksum};

    let hrp = Hrp::parse(hrp).expect("error parsing hrp");

    let result = match mode {
        "bech32" => bech32::encode::<Bech32>(hrp, input),
        "bech32m" => bech32::encode::<Bech32m>(hrp, input),
        "nochecksum" => bech32::encode::<NoChecksum>(hrp, input),
        _ => unimplemented!("only bech32, bech32m and nochecksum are supported"),
    };

    result.unwrap_or_else(|_| panic!("error bech32 encoding using {}", mode))
}

/// Encode the `Hrp` (Human Readable Part) and input into a checksummed lowercase bech32 encoded string.
/// Supports 3 modes:
/// - bech32
/// - bech32m
/// - nochecksum
#[pg_extern(immutable, parallel_safe)]
pub fn bech32_encode_lower(hrp: &str, input: &[u8], mode: &str) -> String {
    use bech32::{Bech32, Bech32m, Hrp, NoChecksum};

    let hrp = Hrp::parse(hrp).expect("error parsing hrp");

    let result = match mode {
        "bech32" => bech32::encode_lower::<Bech32>(hrp, input),
        "bech32m" => bech32::encode_lower::<Bech32m>(hrp, input),
        "nochecksum" => bech32::encode_lower::<NoChecksum>(hrp, input),
        _ => unimplemented!("only bech32, bech32m and nochecksum are supported"),
    };

    result.unwrap_or_else(|_| panic!("error bech32 encoding using {}", mode))
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use super::*;

    #[pg_test]
    fn test_bech32_decode() {
        let bech = bech32_decode("union14qemq0vw6y3gc3u3e0aty2e764u4gs5lnxk4rv");
        assert_eq!(bech.get_by_name("hrp").unwrap(), Some("union"));
        assert_eq!(
            bech.get_by_name::<Vec<u8>>("data").unwrap(),
            Some(vec![
                168, 51, 176, 61, 142, 209, 34, 140, 71, 145, 203, 250, 178, 43, 62, 213, 121, 84,
                66, 159
            ])
        );
    }

    #[pg_test]
    fn test_bech32_encode() {
        let raw = hex::decode("644a2606654a7c0e70bf343ae6b828d3fe448447").unwrap();
        let bech = bech32_encode("union", &raw, "bech32");
        assert_eq!(bech, "union1v39zvpn9ff7quu9lxsawdwpg60lyfpz8pmhfey")
    }

    #[pg_test]
    fn test_bech32_encode2() {
        let raw = hex::decode("7e83d17b15e379b76cbf6966564472e567ccc4a2").unwrap();
        let bech = bech32_encode("union", &raw, "bech32");
        assert_eq!(bech, "union106paz7c4udumwm9ld9n9v3rju4nue39z4nt8tg")
    }

    #[pg_test]
    fn test_encode_bech_from_hex() {
        let result = Spi::get_one::<&str>("SELECT bech32_encode('union'::text, decode('644a2606654a7c0e70bf343ae6b828d3fe448447','hex'), 'bech32'::text)").unwrap();
        assert_eq!(
            result.unwrap(),
            "union1v39zvpn9ff7quu9lxsawdwpg60lyfpz8pmhfey"
        )
    }
}

/// This module is required by `cargo pgrx test` invocations.
/// It must be visible at the root of your extension crate.
#[cfg(test)]
pub mod pg_test {
    pub fn setup(_options: Vec<&str>) {
        // perform one-off initialization when the pg_test framework starts
    }

    pub fn postgresql_conf_options() -> Vec<&'static str> {
        // return any postgresql.conf settings that are required for your tests
        vec![]
    }
}
