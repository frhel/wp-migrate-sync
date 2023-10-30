use std::process::Command;
use std::process::exit;
fn main() {
    // Pick a download method to use [curl, wget] depending on what is available
    let download_methods = check_available_download_methods();
    // Check if we have all the dependencies installed    
    let dependencies = vec!["php", "ssh", "rsync", "bash"];
    if !dependency_checks(&dependencies) { exit(1) }

    // Check if we have WP-CLI installed and install it if we don't
    if !run_wp_cli_check(&download_methods) { exit(1) }    
    
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
    if !cmd_bash("wp core is-installed") == true {        
        println!("Source directory is not a WordPress directory");
        exit(1);
    }
    
    // If we have reached this point, we are in a WordPress directory
    // Now we can start making database checks so we can make sure that
    if !cmd_bash("wp db check") == true {
        println!("No database found");
        exit(1);
    }


    println!("All done!");
    exit(0);
}


fn install_wp_cli(download_methods: &Vec<&str>) -> bool {
    // Check if we have curl or wget installed
    if download_methods.len() == 0 {
        println!("No download methods found, please install curl or wget to continue");
        return false;
    }
    // Download wp-cli.phar
    let wp_dl = cmd_bash(&format!("{} -O https://raw.githubusercontent.com/wp-cli/builds/gh-pages/phar/wp-cli.phar", download_methods[0]));
    let wp_perm = cmd_bash("chmod +x wp-cli.phar");
    let wp_move = cmd_bash("mv wp-cli.phar /usr/local/bin/wp");
    if !wp_move || !wp_dl || !wp_perm {
        if !wp_dl { println!("Failed to download wp-cli.phar"); }
        if !wp_perm { println!("Failed to make wp-cli.phar executable"); }
        if !wp_move { println!("Failed to move wp-cli.phar to /usr/local/bin/wp"); }
        println!("Check if you are running as a user with sudo privileges");
        clean_up_wp_cli_dl();
        return false;
    }
    cmd_bash("wp --version")
}

fn clean_up_wp_cli_dl() {
    let del_dl = cmd_bash("rm wp-cli.phar");
    if !del_dl {
        println!("Failed to clean up wp-cli.phar");
    }
}

fn cmd_bash(command: &str) -> bool {
    let mut success = true;
    let output = Command::new("bash")
        .args(&["-c", command])
        .output()
        .unwrap_or_else(|e| panic!("Failed to execute process: {}", e));
    if !output.status.success() { success = false; }
    success
}

fn run_wp_cli_check(download_methods: &Vec<&str>) -> bool {
    let mut success = cmd_bash("wp -v");
    if !success {
        println!("WP-CLI not found");
        // TODO prompt user to install WP-CLI if they want to continue
        // TODO add a flag to skip prompt - maybe -y or --yes
        println!("Trying to install WP-CLI");
        success = install_wp_cli(&download_methods);
        if success == true {
            println!("WP-CLI installed successfully");
        } else {
            println!("WP-CLI installation failed");
        }
    }
    success
}

fn dependency_checks(dependencies: &Vec<&str>) -> bool {
    let mut missing = Vec::new();
    for dependency in dependencies {
        if cmd_bash(&format!("command -v {}", dependency)) == false {
            missing.push(dependency);
        }
    }
    if missing.len() > 0 {
        missing.iter().for_each(|dep| println!("Missing dependency: {}, - please install it to continue", dep));
    }
    missing.len() == 0
}

// Return a tuple of available download methods
fn check_available_download_methods() -> Vec<&'static str> {
    let expected_methods = vec!["curl", "wget"];
    let mut available_methods: Vec<&str> = Vec::new();
    for method in expected_methods {
        if (cmd_bash(&format!("command -v {}", method))) == false {
            continue;
        } else {
            available_methods.push(method);
        }
    }
    available_methods
}