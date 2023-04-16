use pgx::prelude::*;

pgx::pg_module_magic!();

#[derive(Copy, Clone, PostgresType)]
#[inoutfuncs]
struct Oid {
    data: [u8; 16],
    label: String,
}

impl InOutFuncs for Oid {
    fn input(input: &core::ffi::CStr) -> Self {
        let mut iter = input.to_str().unwrap().split('_');
        let (a, b, c) = (iter.next(), iter.next(), iter.next());

        let mut result = PgVarlena::<MyType>::new();
        result.a = f32::from_str(a.unwrap()).expect("a is not a valid f32");
        result.b = f32::from_str(b.unwrap()).expect("b is not a valid f32");
        result.c = i64::from_str(c.unwrap()).expect("c is not a valid i64");
        result
        Oid
    }

    // Output ourselves as text into the provided `StringInfo` buffer
    fn output(&self, buffer: &mut StringInfo) {
        buffer.push_str(&format!("{},{},{}", self.a, self.b, self.c));
    }
}

#[pg_extern]
fn hello_oid_pg() -> &'static str {
    "Hello, oid_pg"
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use pgx::prelude::*;

    #[pg_test]
    fn test_hello_oid_pg() {
        assert_eq!("Hello, oid_pg", crate::hello_oid_pg());
    }

}

/// This module is required by `cargo pgx test` invocations. 
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
