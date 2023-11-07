use std::process::exit;
use crate::bash_operations::BashOperations;
use crate::download_methods::DownloadMethods;

pub struct WpCliInterface {
    // A vector containing various dependencies that the WP-CLI script needs to run
    // like available download method
    download_method: &'static str,
}

impl WpCliInterface {
    fn default() -> WpCliInterface {
        WpCliInterface {
            download_method: "",
        }
    }

    pub fn new() -> WpCliInterface {
        // TODO Remove - testing only
        WpCliInterface::clean_up_download(&WpCliInterface::default());

        WpCliInterface {
            // Set download methods to the one passed in
            download_method: DownloadMethods::new().first(),
        }
    }

    pub fn is_installed(&self) -> bool {
        let cli = BashOperations::new();
        cli.error_check(&format!("command -v wp"))
    }

    pub fn install(&self) -> bool {
        println!("Trying to install WP-CLI");
        
        // Check if we have curl or wget installed
        println!("Using {} to download WP-CLI", self.download_method);

        let mut cli = BashOperations::new();
        // Download wp-cli.phar
        let download_command = match self.download_method {
            "curl" => "curl -O https://raw.githubusercontent.com/wp-cli/builds/gh-pages/phar/wp-cli.phar",
            "wget" => "wget https://raw.githubusercontent.com/wp-cli/builds/gh-pages/phar/wp-cli.phar",
            _ => "",
        };
        cli.add("download file", download_command);
        cli.add("make wp file executable", "sudo chmod +x wp-cli.phar");
        cli.add("move wp to /usr/local/bin/wp", "sudo mv wp-cli.phar /usr/local/bin/wp");
        cli.run();
        
        if !self.is_installed() {
            eprintln!("Failed to install WP-CLI");
            self.clean_up_download();
            exit(1);        
        }
        
        println!("WP-CLI installed successfully");
        return true;
    }

    fn clean_up_download(&self) {
        let mut cli = BashOperations::new();
        // Check if the file exists before trying to remove it
        if !cli.file_exists("wp-cli.phar") {
            return;
        }

        // TODO remove the operation that removes the executable from /usr/local/bin
        cli.add("remove downloaded file, wp-cli.phar from current directory", "sudo rm wp-cli.phar");
        cli.add("remove executable, wp-cli.phar from /usr/local/bin", "sudo rm /usr/local/bin/wp");
        cli.run();
    }

    pub fn run_preflight_check(&self, source_dir: &str, target_dir: &str) -> bool {
        let mut passed = true;
        // Check that wordpress is installed in both places
        if BashOperations::error_check(&BashOperations::new(), &format!("wp core is-installed --path={}", source_dir)) {
            eprintln!("Source directory is not a WordPress directory. Exiting...");
            passed = false;
        }
        if BashOperations::error_check(&BashOperations::new(), &format!("wp core is-installed --path={}", target_dir)) {
            eprintln!("Target directory is not a WordPress directory. Exiting...");
            passed = false;
        }       

        // Check that a database is available in both places
        if BashOperations::error_check(&BashOperations::new(), &format!("wp db check --path={}", source_dir)) {
            eprintln!("No database found in source directory. Exiting...");
            passed = false;
        }
        if BashOperations::error_check(&BashOperations::new(), &format!("wp db check --path={}", target_dir)) {
            eprintln!("No database found in target directory. Exiting...");
            passed = false;
        }

        // Check that the uploads directory is writable in both places
        let source_uploads = format!("{}/wp-content/uploads", &source_dir);
        let target_uploads = format!("{}/wp-content/uploads", &target_dir);
        if BashOperations::error_check(&BashOperations::new(), &format!("[ -w {} ] && echo 'true' || echo 'false'", source_uploads)) {
            eprintln!("Source uploads directory is not writable. Exiting...");
            passed = false;
        }
        if BashOperations::error_check(&BashOperations::new(), &format!("[ -w {} ] && echo 'true' || echo 'false'", target_uploads)) {
            eprintln!("Target uploads directory is not writable. Exiting...");
            passed = false;
        }

        if !passed { exit(1); }

        true
    }
}