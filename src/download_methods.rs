use std::process::exit;

use crate::bash_operations::BashOperations;

pub struct DownloadMethods {
    // Set default download methods to curl and wget
    pub methods: Vec<&'static str>,
}

impl DownloadMethods {
    fn default() -> DownloadMethods {
        DownloadMethods {
            methods: vec!["curl", "wget"],
        }
    }

    pub fn new() -> DownloadMethods {
        let methods = DownloadMethods::default();
        DownloadMethods {
            methods: methods.install_check(&methods.methods),
        }
    }

    pub fn first(&mut self) -> &'static str {        
        self.methods[0]
    }

    fn install_check(&self, to_iter: &Vec<&'static str>) -> Vec<&'static str> {
        let cli = BashOperations::new();
        let mut collected = Vec::new();
        for dependency in to_iter {
            if cli.error_check(&format!("command -v {}", &dependency)) {
                collected.push(*dependency);
            }
        }

        if self.methods.len() == 0 {
            println!("No download methods found, please install curl or wget to continue");
            exit(1);
        }

        collected
    }
}