use std::process::Command;
use std::process::Output;
use std::process::exit;

struct BashOperations {
    // A vector of tuples containing a message and a command to run
    // The message is used to display to the user what is happening
    // The command is the actual command that is run
    operations: Vec<(&'static str, &'static str)>,
}

impl BashOperations {
    fn new() -> BashOperations {
        BashOperations {
            operations: Vec::new(),
        }
    }

    fn add(&mut self, message: &'static str, operation: &'static str) {
        self.operations.push((message, operation));
    }

    fn run(&self) {
        self.operations.iter().for_each(|op| {
            let output = self.bash(&op.1);
            if self.has_error(&output) {
                //self.print_error(&op.0, &output);
            }
        });
    }

    fn has_error(&self, output: &Output) -> bool {
        // parse output and check for error keywords
        let output_str = String::from_utf8_lossy(&output.stdout).to_ascii_lowercase();
        let err_strings = vec!["error", "failed", "not found", "no such file or directory", "permission denied", "command not found"];
        err_strings.iter().any(|err| output_str.contains(err)) || !output.status.success()
    }

    #[allow(dead_code)]
    fn print_error(&self, command: &str, output: &Output) {
        eprintln!("Output from command: {} - returned an error", command);
        if output.stderr.len() > 0 {
            eprintln!("Line 1 of error output:\n  {}", String::from_utf8_lossy(&output.stderr).split("\n").collect::<Vec<&str>>()[0]);
        } else if output.stdout.len() > 0 {
            eprintln!("Line 1 of error output:\n  {}", String::from_utf8_lossy(&output.stdout).split("\n").collect::<Vec<&str>>()[0]);
        }        
    }

    fn install_check(&self, command: &String) -> bool {
        let output = self.bash(command);
        if self.has_error(&output) {
            // TODO add a flag to skip errors from CLI invocations
            //self.print_error(&command, &output);
        }
        !self.has_error(&output)
    }

    fn install_check_iter(&self, to_iter: &Vec<&'static str>) -> Vec<&'static str> {
        let mut collected = Vec::new();
        for dependency in to_iter {
            if self.install_check(&format!("command -v {}", &dependency)) {
                collected.push(*dependency);
            }
        }
        collected
    }

    fn bash(&self, command: &str) -> Output {
        let output = Command::new("bash")
            .args(&["-c", command])
            .output()
            .unwrap_or_else(|e| panic!("Failed to execute process: {}", e));
        output
    }
}

fn main() {
    let cli = BashOperations::new();

    // Pick a download method to use, from [curl, wget] depending on what is available
    let mut download_methods = vec!["curl", "wget"];
    download_methods = verify_dependencies(&download_methods);
    if download_methods.len() == 0 {
        println!("No download methods found, please install curl or wget to continue");
        exit(1); 
    }

    // Check if we have all the dependencies installed    
    let dependencies = vec!["php", "ssh", "rsync", "bash"];
    let verified_dependencies = verify_dependencies(&dependencies);
    if verified_dependencies.len() == dependencies.len()  {
        println!("Missing dependencies: {:?} - Check that they are all available before continuing.", dependencies);
        exit(1) 
    }

    // Check if we have WP-CLI installed and install it if we don't

    clean_up_wp_cli_dl();
    if !cli.install_check(&format!("wp --version")) {
        println!("WP-CLI not found");
        // TODO prompt user to install WP-CLI if they want to continue
        // TODO add a flag to skip prompt - maybe -y or --yes
        println!("Trying to install WP-CLI");
        if !install_wp_cli(&download_methods) {
            eprintln!("Failed to install WP-CLI");
            exit(1);
        } else {
            println!("WP-CLI installed successfully");
        }
    }
    
    // For now we assume the script is always run in the source directory
    // TODO: Add the arguments to the script so we can start writing actual functionality
    // --source=user@host:/path/to/source *optional - default is current directory
    // --destination=user@host:/path/to/destination - required
    // --exclude=path, path, path, ... or a file with paths to exclude, one per line *optional
    // --include=path, path, path, ... or a file with paths to include, one per line *optional
    // --dry-run *optional - uses the dry-run flag in rsync to show what would be transferred without actually transferring anything
    // --delete *optional - uses the delete flag in rsync to delete files in the destination that are not in the source
    // --sym-uploads=path *optional - writes an .htaccess file in the uploads directory to point to the source uploads http path


    // If we have reached this point, we have successfully installed WP-CLI and made sure that PHP is installed
    // Now we can start making checks to see if we are in a WordPress directory
    if !cli.install_check(&format!("wp core is-installed")) {
        eprintln!("Source directory is not a WordPress directory. Exiting...");
        exit(1);
    }
    
    // If we have reached this point, we are in a WordPress directory
    // Now we can start making database checks so we can make sure that
    if !cli.install_check(&format!("wp db check")) {
        eprintln!("No database found. Exiting...");
        exit(1);
    }


    println!("All done!");
    exit(0);
}


fn install_wp_cli(download_methods: &Vec<&str>) -> bool {
    let download_method = download_methods[0];
    // Check if we have curl or wget installed
    if download_methods.len() == 0 {
        eprintln!("No download methods found, please install curl or wget to continue");
        exit(1);
    } else {
        println!("Using {} to download WP-CLI", download_method);
    }
    let mut ops = BashOperations::new();
    // Download wp-cli.phar
    let download_command = match download_method {
        "curl" => "curl -O https://raw.githubusercontent.com/wp-cli/builds/gh-pages/phar/wp-cli.phar",
        "wget" => "wget https://raw.githubusercontent.com/wp-cli/builds/gh-pages/phar/wp-cli.phar",
        _ => "",
    };
    ops.add("download file", download_command);
    ops.add("make wp file executable", "sudo chmod +x wp-cli.phar");
    ops.add("move wp to /usr/local/bin/wp", "sudo mv wp-cli.phar /usr/local/bin/wp");
    ops.run();
    ops.install_check(&format!("wp --version"))
}

fn clean_up_wp_cli_dl() {
    let mut ops = BashOperations::new();
    ops.add("remove downloaded file, wp-cli.phar from current directory", "sudo rm wp-cli.phar");
    ops.add("remove /usr/local/bin/wp", "sudo rm /usr/local/bin/wp");
    ops.run();
}

fn verify_dependencies(dependencies: &Vec<&'static str>) -> Vec<&'static str> {
    let cli = BashOperations::new();
    let installed = cli.install_check_iter(dependencies);
    installed
}

#[test]
fn test_verify_dependencies() {
    let mut expected_methods = vec!["curl", "wget"];
    let available_methods = verify_dependencies(&expected_methods);
    assert!(available_methods.len() > 0, "Did not find any download methods when we should have");

    expected_methods = vec!["php", "ssh", "rsync", "bash"];
    let available_methods = verify_dependencies(&expected_methods);
    assert!(available_methods.len() > 0, "Did not find all dependencies when we should have");

    expected_methods = vec!["notinstalled"];
    let available_methods = verify_dependencies(&expected_methods);
    assert!(available_methods.len() == 0, "Found dependencies when we should not have");
}