pub(crate) use crate::halt;
pub(crate) use crate::tty::*;


#[macro_use]
pub(crate) mod macros {
    macro_rules! module {
        ($modue:tt) => {
            print("\n\n################ ");
            print($modue);
            print(" ################");
        };
        ($modue:tt,$category:tt) => {
            print("\n\n################ ");
            print($modue);
            print(" #### ");
            print($category);
            print(" ################");
        };
    }

    macro_rules! scenario {
        ($title:tt) => {
            print("\n\n[TEST] ");
            print($title);
        };
    }

    macro_rules! fail_scenario {
        ($title:tt) => {
            print("\n\n[TEST](FAIL) ");
            print($title);
        };
    }

    macro_rules! show_hex {
        ($description:tt,$datum:tt) => {
            if env!("TESTS_DEBUG") == "1" {
                print("\n|> ");
                print($description);
                print(": ");
                print_hex($datum);
            }
        };
        ($description:tt,$datum:expr) => {
            if env!("TESTS_DEBUG") == "1" {
                print("\n|> ");
                print($description);
                print(": ");
                print_hex($datum);
            }
        };
        ($datum:tt) => {
            if env!("TESTS_DEBUG") == "1" {
                print("\n|> ");
                print_hex($datum);
            }
        };
    }

    macro_rules! test_passed {
        () => {
            print("\n|==> PASSED");
        };
    }

    macro_rules! test {
        ($condition:expr,$message:tt) => {
            match $condition {
                true => print("\n|==> PASSED"),
                false => {
                    print("\n|==> FAILED: ");
                    print($message);
                    loop { halt(); }
                },
            }
        };
    }

    macro_rules! check {
        ($condition:expr,$message:tt) => {
            if !$condition {
                print("\n|==> FAILED: ");
                print($message);
                loop { halt(); }
            }
        };
    }

    macro_rules! assume {
        ($condition:expr,$message:tt) => {
            if !$condition {
                print("\n|x=> ASSUMPTION FAILURE: ");
                print($message);
                loop { halt(); }
            }
        };
    }

    macro_rules! wait {
        () => {
            if env!("TESTS_DEBUG") == "1" {
                for _ in 0..500_000 { unsafe { core::arch::asm!("nop") } }
            }
        };
    }
}


pub(crate) use {
    module,
    scenario,
    fail_scenario,
    show_hex,
    test_passed,
    test,
    check,
    assume,
    wait,
};

pub(crate) trait TestMethods {
    fn fake_new() -> Self;
}
