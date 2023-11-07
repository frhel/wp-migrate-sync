use std::process::exit;

use crate::bash_operations::BashOperations;
use crate::wp_cli_interface::WpCliInterface;

mod download_methods;
mod bash_operations;
mod wp_cli_interface;



fn main() {
    let cli = BashOperations::new();
    let wp_cli = WpCliInterface::new();

    // System preflight checks to make sure we have everything we need to run the script
    system_preflight_checks(&cli);

    // Check if we have WP-CLI installed and install it if we don't
    if !wp_cli.is_installed() { wp_cli.install(); }


    
    // For now we assume the script is always run in the source directory
    // TODO: Add the arguments to the script so we can start writing actual functionality
    // --source=user@host:/path/to/source *optional - default is current directory
    // --destination=user@host:/path/to/destination - required
    // --exclude=path, path, path, ... or a file with paths to exclude, one per line *optional
    // --include=path, path, path, ... or a file with paths to include, one per line *optional
    // --dry-run *optional - uses the dry-run flag in rsync to show what would be transferred without actually transferring anything
    // --delete *optional - uses the delete flag in rsync to delete files in the destination that are not in the source
    // --sym-uploads=path *optional - writes an .htaccess file in the uploads directory to point to the source uploads http path

    // Check whether a wpms.conf file was passed in as an argument
    // If it was, we can use that to get the source and destination directories
    // along with any other options that we need to use
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
