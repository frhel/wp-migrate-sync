use std::process::Command;
use std::process::exit;
fn main() {
    let download_methods = check_available_download_methods();
    if run_command("php --version") == false {
        println!("PHP not found, please install it");
        exit(1);
    }
    if run_wp_cli_check(&download_methods) == false {
        println!("WP-CLI not found, please install it");
        exit(1);
    }

    // If we have reached this point, we have successfully installed WP-CLI and made sure that PHP is installed
    // Now we can start making checks to see if we are in a WordPress directory
    if !run_command("wp core is-installed") == true {        
        println!("Source directory is not a WordPress directory");
        exit(1);
    }
    
    // If we have reached this point, we are in a WordPress directory
    // Now we can start making database checks so we can make sure that
    if !run_command("wp db check") == true {
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
    let wp_dl = run_command(&format!("{} -O https://raw.githubusercontent.com/wp-cli/builds/gh-pages/phar/wp-cli.phar", download_methods[0]));
    let wp_perm = run_command("chmod +x wp-cli.phar");
    let wp_move = run_command("sudo mv wp-cli.phar /usr/local/bin/wp");
    if !wp_move || !wp_dl || !wp_perm {
        if !wp_dl { println!("Failed to download wp-cli.phar"); }
        if !wp_perm { println!("Failed to make wp-cli.phar executable"); }
        if !wp_move { println!("Failed to move wp-cli.phar to /usr/local/bin/wp"); }
        println!("Check if you are running this script as a user with sudo privileges");
        clean_up_wp_cli_dl();
        return false;
    }
    run_command("wp --version")
}

fn clean_up_wp_cli_dl() {
    let del_dl = run_command("rm wp-cli.phar");
    if !del_dl {
        println!("Failed to clean up wp-cli.phar");
    }
}

fn run_command(command: &str) -> bool {
    let output = Command::new("bash")
        .args(&["-c", command])
        .output()
        .unwrap_or_else(|e| panic!("Failed to execute process: {}", e));
    if output.status.success() {
        return true;
    }
    false
}

fn run_wp_cli_check(download_methods: &Vec<&str>) -> bool {
    let mut wp_cli = run_command("wp --version");
    if wp_cli == true {
        return true;
    } else {
        if wp_cli == false {
            println!("WP-CLI not found");
            println!("Installing WP-CLI");
            wp_cli = install_wp_cli(&download_methods);
            if wp_cli == true {
                println!("WP-CLI installed successfully");
                return true;
            } else {
                println!("WP-CLI installation failed");
            }
        }
    }
    false
}

// Return a tuple of available download methods
fn check_available_download_methods() -> Vec<&'static str> {
    let expected_methods = vec!["curl", "wget"];
    let mut available_methods: Vec<&str> = Vec::new();
    for method in expected_methods {
        if (run_command(&format!("command -v {}", method))) == false {
            continue;
        } else {
            available_methods.push(method);
        }
    }
    available_methods
}