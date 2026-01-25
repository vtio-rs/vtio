mod common;

struct TestSuite {
    name: String,
    input: String,
    output_path: String,
    title: String,
}

fn discover_test_suites() -> Vec<TestSuite> {
    vec![
        TestSuite {
            name: "keyboard_input".to_string(),
            input: include_str!("keyboard_input.txt").to_string(),
            output_path: "tests/keyboard_result.md".to_string(),
            title: "Keyboard Input".to_string(),
        },
        TestSuite {
            name: "utf8_handling".to_string(),
            input: include_str!("utf8_handling.txt").to_string(),
            output_path: "tests/utf8_result.md".to_string(),
            title: "UTF-8 Handling".to_string(),
        },
        TestSuite {
            name: "bracketed_paste".to_string(),
            input: include_str!("bracketed_paste.txt").to_string(),
            output_path: "tests/bracketed_paste_result.md".to_string(),
            title: "Bracketed Paste".to_string(),
        },
        TestSuite {
            name: "control_keys".to_string(),
            input: include_str!("control_keys.txt").to_string(),
            output_path: "tests/control_keys_result.md".to_string(),
            title: "Control Keys".to_string(),
        },
        TestSuite {
            name: "alt_keys".to_string(),
            input: include_str!("alt_keys.txt").to_string(),
            output_path: "tests/alt_keys_result.md".to_string(),
            title: "Alt Keys".to_string(),
        },
        TestSuite {
            name: "kitty_keyboard".to_string(),
            input: include_str!("kitty_keyboard.txt").to_string(),
            output_path: "tests/kitty_keyboard_result.md".to_string(),
            title: "Kitty Keyboard Protocol".to_string(),
        },
    ]
}

pub fn main() {
    let filter = std::env::args().nth(1).unwrap_or_default();
    let suites = discover_test_suites();

    // If filter is provided, only run matching suite
    let suites_to_run: Vec<_> = if filter.is_empty() {
        suites
    } else {
        suites
            .into_iter()
            .filter(|s| s.name.contains(&filter))
            .collect()
    };

    if suites_to_run.is_empty() {
        eprintln!("No test suites match filter: {filter}");
        std::process::exit(1);
    }

    for suite in suites_to_run {
        println!("\n=== Running test suite: {} ===\n", suite.title);

        common::run_tests(common::TestConfig {
            input_file: &suite.input,
            output_file: &suite.output_path,
            title: &suite.title,
            filter: "",
        });

        println!("\n=== {} completed ===\n", suite.title);
    }
}
