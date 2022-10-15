use std::{fs, io, process::Command};

const GOOGLE_DIR_APPEND_PATH: &str = "/Library/Application Support/Google";
const FILE_TEMPLATES_APPEND_PATH: &str = "/fileTemplates";
const AS_INSTALLATION_FOLDER_PREFIX: &str = "AndroidStudio";

const SHARED_TEMPLATES_PATH: &str = "SharedTemplates";

fn main() {
    // Process shared templates
    let shared_templates_folder =
        fs::read_dir(SHARED_TEMPLATES_PATH).expect("No \"SharedTemplates\" folder found");

    let shared_templates: Vec<String> = shared_templates_folder
        .filter(|v| v.is_ok())
        .map(|v| {
            v.unwrap()
                .path()
                .to_str()
                .map(|str_value| String::from(str_value))
        })
        .filter(|v| v.is_some())
        .map(|v| v.unwrap())
        .collect();

    // Get AS installation folder
    let mut templates_path = String::new();

    match home::home_dir() {
        Some(path) => templates_path.push_str(path.to_str().unwrap()),
        None => panic!("Impossible to get your home dir"),
    }

    templates_path.push_str(GOOGLE_DIR_APPEND_PATH);

    let google_dir =
        fs::read_dir(templates_path).expect("Failed to open Android Studio installation folder");

    let installation_folders: Vec<String> = google_dir
        .filter(|v| v.is_ok())
        .map(|v| {
            v.unwrap()
                .path()
                .to_str()
                .map(|str_value| String::from(str_value))
        })
        .filter(|v| v.is_some())
        .map(|v| v.unwrap())
        .filter(|v| v.contains(AS_INSTALLATION_FOLDER_PREFIX))
        .collect();

    let as_path: String;

    match installation_folders.len() {
        0 => panic!("No Android Studio installation found"),
        1 => {
            as_path = installation_folders
                .get(0)
                .expect("Failed to retrieve installation folder")
                .clone()
        }
        // Several AS installations
        _ => {
            println!("Several AS installations found");

            let mut display_index = 1;
            for path in &installation_folders {
                println!("{}: {}", display_index, name_from_path(path.as_str()));
                display_index += 1;
            }

            println!("Enter installation number (number before semicolon):");

            let mut input_text = String::new();
            io::stdin()
                .read_line(&mut input_text)
                .expect("Failed to read from stdin");

            match input_text.trim().parse::<usize>() {
                Ok(i) => {
                    as_path = installation_folders
                        .get(i - 1)
                        .expect("Invalid index")
                        .clone()
                }
                Err(_) => panic!("Invalid input!"),
            };
        }
    }

    // Check for fileTemplates folder and create one if absent
    let mut as_templates_path = as_path.clone();
    as_templates_path.push_str(FILE_TEMPLATES_APPEND_PATH);

    if fs::read_dir(as_templates_path.as_str()).is_err() {
        println!("No  \"fileTemplates\" folder. Creating one...");
        Command::new("mkdir")
            .arg(&as_templates_path)
            .spawn()
            .expect("Failed to open/create \"fileTemplates\" folder");
    }

    // Copy templates
    for template in shared_templates {
        let mut target_path = as_templates_path.clone();
        target_path.push('/');
        target_path.push_str(name_from_path(template.as_str()));

        Command::new("cp")
            .arg(&template)
            .arg(target_path)
            .spawn()
            .expect(format!("Failed to copy a template {}", &template).as_str());
    }

    println!("Process complete! Restart AS for changes to take effect.")
}

fn name_from_path(src: &str) -> &str {
    src.split('/').last().unwrap()
}
