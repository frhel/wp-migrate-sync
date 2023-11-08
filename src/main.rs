use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use std::process::exit;

use crate::bash_operations::BashOperations;
use crate::wp_cli_interface::WpCliInterface;

mod download_methods;
mod bash_operations;
mod wp_cli_interface;

#[derive(Debug)]
struct Config {
    source: String,
    destination: String,
    exclude: String,
    dry_run: bool,
    delete: bool,
    sym_uploads: String,
}

impl Config {
    fn new() -> Config {
        Config {            
            source: String::from("./"),
            destination: String::from("./"),
            exclude: String::from(""),
            dry_run: false,
            delete: false,
            sym_uploads: String::from(""),
        }
    }

    #[allow(dead_code)]
    fn interactive() -> Config {
        let cli = BashOperations::new();
        let source = cli.prompt_input("Enter the source directory, < default none > (<user@host:port>:path/to/source): ");
        let destination = cli.prompt_input("Enter the destination directory - required, (<user@host:port>:path/to/destination): ");
        let exclude = cli.prompt_input("Enter the directories to exclude < default none >, separate by whitespace: ");
        let dry_run = cli.prompt_bool("Dry run? Run operation without transferring files... (y/n): ");
        let delete = cli.prompt_bool("Delete files in destination that are not in source? (y/n): ");
        let sym_uploads = cli.prompt_input("Symlink uploads file to a different folder on target? < default none > (path/to/destination): ");

        Config {
            source: source,
            destination: destination,
            exclude: exclude,
            dry_run: dry_run,
            delete: delete,
            sym_uploads: sym_uploads,
        }
    }

    fn from_args(&mut self, args: &Vec<&str>) {
        let mut args_iter = args.iter();
        args_iter.next(); // Skip the first argument which is the script name
        while let Some(arg) = args_iter.next() {
            // Split the argument if it contains an equals sign
            // This is so we can pass in arguments like --source=user@host:/path/to/source
            // and --destination=user@host:/path/to/destination
            let arg = arg.split("=").collect::<Vec<&str>>();
            let value = arg[1].to_string();
            let arg = arg[0].to_string();
            match arg.as_str() {
                "--source" => {
                    self.source = value;
                },
                "--destination" => {
                    self.destination = value;
                },
                "--exclude" => {
                    self.exclude = value;
                },
                "--dry-run" => {
                    self.dry_run = true;
                },
                "--delete" => {
                    self.delete = true;
                },
                "--sym-uploads" => {
                    self.sym_uploads = value;
                },
                _ => {
                    println!("Unknown argument: {}", arg);
                },
            }
        }
        
    }

    fn file(&self, file: &str) -> Config {
        // read the file
        let file = File::open(file).expect("Unable to open file");
        let mut buf_reader = BufReader::new(file);

        // parse the file
        let mut config = Config::new();
        let mut line = String::new();
        while let Ok(bytes_read) = buf_reader.read_line(&mut line) {
            if bytes_read == 0 { break; }
            let line = line.trim().to_string();
            let line = line.split("=").collect::<Vec<&str>>().iter().map(|x| x.trim()).collect::<Vec<&str>>();
            let value = line[1].to_string();
            let mut line = line[0].to_string();
            match line.as_str() {
                "source" => {
                    config.source = value;
                },
                "destination" => {
                    config.destination = value;
                },
                "exclude" => {
                    config.exclude = value;
                },
                "dry-run" => {
                    config.dry_run = true;
                },
                "delete" => {
                    config.delete = true;
                },
                "sym-uploads" => {
                    config.sym_uploads = value;
                },
                _ => {
                    println!("Unknown argument: {}", line);
                },
            }
            line.clear();
        }

        // return the config
        config


    }



}



fn main() {
    let cli = BashOperations::new();
    let wp_cli = WpCliInterface::new();

    // System preflight checks to make sure we have everything we need to run the script
    system_preflight_checks(&cli);

    // Check if we have WP-CLI installed and install it if we don't
    if !wp_cli.is_installed() { wp_cli.install(); }
    // TODO add an update check and prompt to update if there is a new version available

    
    // For now we assume the script is always run in the source directory
    // TODO: Add the arguments to the script so we can start writing actual functionality
    // --config *optional - default is wpms.conf or cli arguments
    // --source=user@host:/path/to/source *optional - default is current directory
    // --destination=user@host:/path/to/destination - required
    // --exclude=path, path, path, ... or a file with paths to exclude, one per line *optional
    // --dry-run *optional - uses the dry-run flag in rsync to show what would be transferred without actually transferring anything
    // --delete *optional - uses the delete flag in rsync to delete files in the destination that are not in the source
    // --sym-uploads=path *optional - writes an .htaccess file in the uploads directory to point to the source uploads http path

    // Check whether a wpms.conf file was passed in as an argument or if it exists in the current directory
    // If it was, we can use that to get the source and destination directories
    // along with any other options that we need to use
    
    // Read passed in arguments
    let mut config = Config::new();
    //let args = std::env::args().collect::<Vec<String>>();
    let args = vec!["./wp-migrate-sync", "--source=./here", "--destination=./there", "--exclude=./wp-content/uploads/"];
    if args.len() > 1 {    
        config.from_args(&args);
    } else if cli.file_exists("wpms.conf", "") {
        config = Config::file(&config, "wpms.conf");
    } else {
        // If we don't have a config file, we can prompt the user for the options
        config = Config::interactive();
    }
    println!("Config: {:?}", config);
    // Print the arguments passed in
    println!("Arguments passed in: {:?}", &args);

    let source_dir = ".";
    let target_dir = ".";



    // If we have reached this point, we have successfully installed WP-CLI and made sure that PHP is installed
    // Now we can start making checks to see if we are in a WordPress directory, whether we have a wp-config.php file
    // and whether we have a database connection to the WordPress database
    // If we have all of these things, we can start the sync process...?
    wp_cli.run_preflight_check(source_dir, target_dir);


    println!("All done!");
    exit(0);
}

fn system_preflight_checks(cli: &BashOperations) {
    // Check if we have all the dependencies installed    
    let dependencies = vec!["php", "ssh", "rsync", "bash"];
    let verified_dependencies = cli.install_check(&dependencies);
    if verified_dependencies.len() < dependencies.len()  {
        println!("Missing dependencies: {:?} - Check that they are all available before continuing.", dependencies);
        exit(1) 
    }
}
