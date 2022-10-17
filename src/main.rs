use regex::Regex;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::{env, fs, process::Command};

const GOOGLE_DIR_APPEND_PATH: &str = "/Library/Application Support/Google";
const FILE_TEMPLATES_APPEND_PATH: &str = "/fileTemplates";
const OPTIONS_APPEND_PATH: &str = "/options";
const AS_INSTALLATION_FOLDER_PREFIX: &str = "AndroidStudio";
const SHARED_TEMPLATES_XML: &str = "shared_templates.xml";
const FILE_TEMPLATE_SETTINGS_XML: &str = "file.template.settings.xml";

const TEMPLATE_NAME_REGEX: &str = r#"<template name="(.+?)""#;
const SINGLE_LINE_TEMPLATE_REGEX: &str = r#"<template name="(.+?)".+?/>"#;
const MULTI_LINE_TEMPLATE_START_REGEX: &str = r#"<template name="(.+?)".+?">"#;
const MULTI_LINE_TEMPLATE_END_REGEX: &str = r"</template>";
const TEMPLATES_BLOCK_END: &str = r"</default_templates>";
const SHARED_TEMPLATES_REGEX: &str = r"<shared_templates>\n([\S\s]+?)</shared_templates>";

const TEMPLATE_SETTINGS_START: &str =
 "<application>\n\t<component name=\"ExportableFileTemplateSettings\">\n\t\t<default_templates>\n";
const TEMPLATE_SETTINGS_END: &str = "\t</default_templates>\n\t</component>\n</application>";

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Invalid arguments")
    }

    let shared_tp_path = args.get(1).expect("Invalid arguments provided").as_str();

    let as_path = get_as_installation_path().unwrap();

    copy_config(&shared_tp_path, &as_path);

    let shared_templates = load_templates(shared_tp_path);
    let as_templates_path = get_target_templates_path(&as_path);

    copy_templates(&as_templates_path, shared_templates);

    println!("Process complete! Restart AS for changes to take effect.")
}

fn copy_config(shared_tp_path: &str, as_path: &str) {
    let mut names: Vec<String> = Vec::new();

    let mut shared_tp_config_path = String::from(shared_tp_path);
    if !shared_tp_config_path.ends_with('/') {
        shared_tp_config_path.push('/');
    }
    shared_tp_config_path.push_str(SHARED_TEMPLATES_XML);

    let src_string = fs::read_to_string(&shared_tp_config_path).expect("shared_templates.xml");

    let shared_regex = Regex::new(SHARED_TEMPLATES_REGEX).unwrap();
    let template_name_regex = Regex::new(TEMPLATE_NAME_REGEX).unwrap();

    let templates_to_insert = shared_regex
        .captures(&src_string)
        .unwrap()
        .get(1)
        .unwrap()
        .as_str();

    for cp in template_name_regex.captures_iter(&templates_to_insert) {
        names.push(String::from(cp.get(1).unwrap().as_str()));
    }

    let mut as_tp_settings_path = String::from(as_path);
    as_tp_settings_path.push_str(OPTIONS_APPEND_PATH);
    as_tp_settings_path.push('/');
    as_tp_settings_path.push_str(FILE_TEMPLATE_SETTINGS_XML);

    let mut result_buff = String::new();

    if let Ok(target_file) = File::open(&as_tp_settings_path) {
        let target_reader = BufReader::new(target_file);

        let single_line_tp = Regex::new(SINGLE_LINE_TEMPLATE_REGEX).unwrap();
        let multi_line_tp_start = Regex::new(MULTI_LINE_TEMPLATE_START_REGEX).unwrap();
        let multi_line_tp_end = Regex::new(MULTI_LINE_TEMPLATE_END_REGEX).unwrap();

        let mut inside_block = false;
        let mut ignore_current_block = true;

        for line in target_reader.lines() {
            if line.is_err() {
                break;
            }

            let line_text = line.unwrap();

            let append_line = if multi_line_tp_end.is_match(&line_text) {
                inside_block = false;
                !ignore_current_block
            } else if inside_block {
                !ignore_current_block
            } else if multi_line_tp_start.is_match(&line_text) {
                let name = multi_line_tp_start
                    .captures(&line_text)
                    .unwrap()
                    .get(1)
                    .unwrap();

                inside_block = true;
                ignore_current_block = names.contains(&String::from(name.as_str()));
                !ignore_current_block
            } else if single_line_tp.is_match(&line_text) {
                let name = single_line_tp.captures(&line_text).unwrap().get(1).unwrap();
                !names.contains(&String::from(name.as_str()))
            } else {
                if line_text.trim() == TEMPLATES_BLOCK_END {
                    result_buff.push_str(&templates_to_insert);
                }
                true
            };

            if append_line {
                result_buff.push_str(&line_text);
                result_buff.push('\n');
            }
        }
    } else {
        result_buff.push_str(TEMPLATE_SETTINGS_START);
        result_buff.push_str(&templates_to_insert);
        result_buff.push_str(TEMPLATE_SETTINGS_END);
    }

    let mut target_settings_file = File::create(&as_tp_settings_path).unwrap();
    target_settings_file
        .write_all(result_buff.as_bytes())
        .expect("Failed to update file.template.settings.xml");
}

fn copy_templates(as_templates_path: &str, shared_templates: Vec<String>) {
    for template in shared_templates {
        let mut target_path = String::from(as_templates_path);
        target_path.push('/');
        target_path.push_str(name_from_path(template.as_str()));

        Command::new("cp")
            .arg(&template)
            .arg(target_path)
            .spawn()
            .expect(format!("Failed to copy a template {}", &template).as_str());
    }
}

fn load_templates(destination: &str) -> Vec<String> {
    let shared_templates_folder = fs::read_dir(destination).expect("No templates folder found");

    shared_templates_folder
        .filter(|v| v.is_ok())
        .map(|v| {
            v.unwrap()
                .path()
                .to_str()
                .map(|str_value| String::from(str_value))
        })
        .filter(|v| v.is_some())
        .map(|v| v.unwrap())
        .filter(|v| !v.contains(SHARED_TEMPLATES_XML))
        .collect()
}

fn get_as_installation_path() -> Result<String, &'static str> {
    let mut templates_path = String::new();

    match home::home_dir() {
        Some(path) => templates_path.push_str(path.to_str().unwrap()),
        None => return Err("Failed to retrieve home path"),
    }

    templates_path.push_str(GOOGLE_DIR_APPEND_PATH);

    let google_dir = match fs::read_dir(templates_path) {
        Ok(dir) => dir,
        Err(..) => return Err("Failed to open AndroidStudio installation folder"),
    };

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
        0 => return Err("No Android Studio installation found"),
        1 => {
            as_path = match installation_folders.get(0) {
                Some(path) => path.clone(),
                None => return Err("Failed to retrieve installation folder"),
            }
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
            match io::stdin().read_line(&mut input_text) {
                Ok(..) => {}
                Err(..) => return Err("Failed to read from stdin"),
            }

            match input_text.trim().parse::<usize>() {
                Ok(i) => {
                    as_path = match installation_folders.get(i - 1) {
                        Some(path) => path.clone(),
                        None => return Err("Failed to retrieve installation folder"),
                    }
                }
                Err(_) => return Err("Invalid input!"),
            };
        }
    }

    Ok(as_path)
}

fn get_target_templates_path(as_path: &str) -> String {
    let mut as_templates_path = String::from(as_path);
    as_templates_path.push_str(FILE_TEMPLATES_APPEND_PATH);

    if fs::read_dir(as_templates_path.as_str()).is_err() {
        println!("No  \"fileTemplates\" folder. Creating one...");
        Command::new("mkdir")
            .arg(&as_templates_path)
            .spawn()
            .expect("Failed to open/create \"fileTemplates\" folder");
    }

    as_templates_path
}

fn name_from_path(src: &str) -> &str {
    src.split('/').last().unwrap()
}
