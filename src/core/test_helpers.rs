#[macro_export]
macro_rules! assert_success {
    ($parser:expr, $input:expr, $value:expr$(, $pos:expr$(,)?)?) => {
        let result = $parser.parse($input);
        if result.is_err() {
            panic!(
                "In parse_on, expected success, but got {:?}",
                result.unwrap_err()
            );
        }
        let result = result.unwrap();
        assert_that!(result.value, eq($value), "in parse_on");
        let set_pos: usize = $input.len(); // default if not provided
        $( let set_pos: usize = $pos; )?

        assert_that!(result.context.position, eq(set_pos), "in parse_on");

        let pos = $parser.fast_parse_on($input.chars().collect::<Vec<_>>().into(), 0);
        if pos.is_none() {
            panic!("In fast_parse_on, expected position after successful parse, but got None");
        }
        let pos = pos.unwrap();
        assert_that!(pos, eq(set_pos), "in fast_parse_on");
    };
}

#[macro_export]
macro_rules! assert_failure {
    ($parser:expr, $input:expr, $message:expr$(, $pos:expr$(,)?)?) => {
        let result = $parser.parse($input);
        if result.is_ok() {
            panic!("Expected failure, but got success {:?}", result.unwrap());
        }
        let failure = result.unwrap_err();
        assert_that!(failure.message, eq($message));

        let set_pos: usize = $input.len();
        $( let set_pos: usize = $pos; )?
        assert_that!(failure.context.position, eq(set_pos));

        let pos = $parser.fast_parse_on($input.chars().collect::<Vec<_>>().into(), 0);
        assert_that!(pos, eq(None));
    };
}
