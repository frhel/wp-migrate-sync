use std::process::Command;
use std::process::exit;
fn main() {
    if is_php_installed() == false {
        println!("PHP not found, please install it");
        exit(1);
    }
    let download_methods = check_available_download_methods();
    if wp_cli_handler(&download_methods) == false {
        println!("WP-CLI not found, please install it");
        exit(1);
    }

    // If we have reached this point, we have successfully installed WP-CLI and made sure that PHP is installed
    // Now we can start making checks to see if we are in a WordPress directory
    let output = Command::new("bash")
        .args(&["-c", "wp core is-installed"])
        .output()
        .unwrap_or_else(|e| panic!("Failed to execute process: {}", e));
    if !output.status.success() {
        println!("Source directory is not a WordPress directory");
        exit(1);
    }
    // If we have reached this point, we are in a WordPress directory
    // Now we can start making checks to see if we have a database
    let output = Command::new("bash")
        .args(&["-c", "wp db check"])
        .output()
        .unwrap_or_else(|e| panic!("Failed to execute process: {}", e));
    if !output.status.success() {
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
    let output = Command::new("bash")
        .args(&["-c", &format!("{} -O https://raw.githubusercontent.com/wp-cli/builds/gh-pages/phar/wp-cli.phar", download_methods[0])])
        .output()
        .unwrap_or_else(|e| panic!("Failed to execute process: {}", e));    
    // Check if we have successfully installed the wp-cli utility
    if output.status.success() {
        let output = Command::new("bash")
            .args(&["-c", "chmod +x wp-cli.phar && sudo mv wp-cli.phar /usr/local/bin/wp"])
            .output()
            .unwrap_or_else(|e| panic!("Failed to execute process: {}", e));
        if output.status.success() {
            return is_wp_installed();
        }
    }
    false          
}

fn is_php_installed() -> bool {
    let output = Command::new("bash")
        .args(&["-c", "php --version"])
        .output()
        .unwrap_or_else(|e| panic!("Failed to execute process: {}", e));
    if output.status.success() {
        return true;
    }
    false
}

fn is_wp_installed() -> bool {
    let output = Command::new("bash")
        .args(["-c", "wp --version"])
        .output()
        .unwrap_or_else(|e| panic!("Failed to execute process: {}", e));
    if output.status.success() {
        return true;
    }
    false
}

fn wp_cli_handler(download_methods: &Vec<&str>) -> bool {
    let mut wp_cli = is_wp_installed();
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
        let output = Command::new("bash")
            .args(&["-c", &format!("command -v {}", method)])
            .output()
            .unwrap_or_else(|e| panic!("Failed to execute process: {}", e));
        if output.status.success() {
            available_methods.push(method);
        }
    }
    available_methods
}