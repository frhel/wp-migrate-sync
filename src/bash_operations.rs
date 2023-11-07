use std::process::{Command, Output};

pub struct BashOperations {
    // A vector of tuples containing a message and a command to run
    // The message is used to display to the user what is happening
    // The command is the actual command that is run
    operations: Vec<(&'static str, &'static str)>,
}

impl BashOperations {
    pub fn new() -> BashOperations {
        BashOperations {
            operations: Vec::new(),
        }
    }

    pub fn add(&mut self, message: &'static str, operation: &'static str) {
        self.operations.push((message, operation));
    }

    pub fn run(&self) -> bool {
        let mut success = true;
        self.operations.iter().for_each(|op| {
            let output = self.bash(&op.1);
            if self.has_error(&output) {
                // TODO add a flag to skip errors from CLI invocations
                // TODO enable this function to be used to print errors when verbose flag is set
                //self.print_error(&op.0, &output);
                success = false;
            }
        });
        if !success {
            return false;
        }
        true
    }

    pub fn has_error(&self, output: &Output) -> bool {
        // parse output and check for error keywords
        let output_str = String::from_utf8_lossy(&output.stdout).to_ascii_lowercase();
        let err_strings = vec!["error", "failed", "not found", "no such file or directory", "permission denied", "command not found"];
        err_strings.iter().any(|err| output_str.contains(err)) || !output.status.success()
    }

    // TODO enable this function to be used to print errors when verbose flag is set
    #[allow(dead_code)]
    pub fn print_error(&self, command: &str, output: &Output) {
        eprintln!("Output from command: {} - returned an error", command);
        if output.stderr.len() > 0 {
            eprintln!("Line 1 of error output:\n  {}", String::from_utf8_lossy(&output.stderr).split("\n").collect::<Vec<&str>>()[0]);
        } else if output.stdout.len() > 0 {
            eprintln!("Line 1 of error output:\n  {}", String::from_utf8_lossy(&output.stdout).split("\n").collect::<Vec<&str>>()[0]);
        }       
    }

    pub fn file_exists(&self, file: &str) -> bool {
        let output = self.bash(&format!("[ -f {} ] && echo 'true' || echo 'false'", file));
        let output_str = String::from_utf8_lossy(&output.stdout).to_ascii_lowercase();
        output_str.contains("true")
    }

    pub fn error_check(&self, command: &String) -> bool {
        let output = self.bash(command);
        if self.has_error(&output) {
            // TODO add a flag to skip errors from CLI invocations
            //self.print_error(&command, &output);
        }
        !self.has_error(&output)
    }

    pub fn install_check(&self, to_iter: &Vec<&'static str>) -> Vec<&'static str> {
        let mut collected = Vec::new();
        for dependency in to_iter {
            if self.error_check(&format!("command -v {}", &dependency)) {
                collected.push(*dependency);
            }
        }
        collected
    }

    pub fn bash(&self, command: &str) -> Output {
        let output = Command::new("bash")
            .args(&["-c", command])
            .output()
            .unwrap_or_else(|e| panic!("Failed to execute process: {}", e));
        output
    }
}


#[test]
fn verify_cli_new() {
    let cli = BashOperations::new();
    assert_eq!(cli.operations.len(), 0);
}
#[test]
fn verify_cli_add() {
    let mut cli = BashOperations::new();
    cli.add("test command", "command -v ls");
    assert_eq!(cli.operations.len(), 1);
    cli.add("test command", "command -v cd");
    assert_eq!(cli.operations.len(), 2);
}
#[test]
fn verify_cli_file_exists() {
    let cli = BashOperations::new();
    assert_eq!(cli.file_exists("src/main.rs"), true);
    assert_eq!(cli.file_exists("src/main.rss"), false);
}

#[test]
fn verify_cli_error_check() {
    let cli = BashOperations::new();
    assert_eq!(cli.error_check(&String::from("command -v ls")), true);
    assert_eq!(cli.error_check(&String::from("command -v bad_operation")), false);
}
#[test]
fn verify_cli_has_error() {
    let cli = BashOperations::new();
    let output = cli.bash("command -v ls");
    assert_eq!(cli.has_error(&output), false);
    let output = cli.bash("command -v bad_operation");
    assert_eq!(cli.has_error(&output), true);
}
#[test]
fn verify_cli_bash() {
    let cli = BashOperations::new();
    let output = cli.bash("command -v cd");
    assert_eq!(cli.has_error(&output), false);
    let output = cli.bash("command -v bad_operation");
    assert_eq!(cli.has_error(&output), true);
}
#[test]
fn verify_cli_install_check() {
    let cli = BashOperations::new();
    let dependencies = vec!["ls", "cd", "pwd"];
    let verified_dependencies = cli.install_check(&dependencies);
    assert_eq!(verified_dependencies.len(), dependencies.len());
    let dependencies = vec!["ls", "cd", "pwd", "bad_operation"];
    let verified_dependencies = cli.install_check(&dependencies);
    assert_eq!(verified_dependencies.len() == dependencies.len(), false);
}
#[test]
fn verify_cli_run() {
    let mut cli = BashOperations::new();
    cli.add("test command", "command -v ls");
    let success = cli.run();
    assert_eq!(success, true);
    cli.add("test command", "command -v bad_operation");
    let success = cli.run();
    assert_eq!(success, false);
}
