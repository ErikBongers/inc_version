use std::{env, fs, io};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use clap::{arg, Arg, ArgAction, Command};
use path_absolutize::*;

fn main() {
    let matches = Command::new("inc_version")
        .version("1.0")
        .about("Increases a semantic version in a given file.\nIf trigger files are specified, the version only gets increased when the trigger files are more recent than the version file.\nE.g., if you specify an exe file as a trigger, the version only gets increased after the exe was successfully built.")
        .arg(Arg::new("version_file"))
        .arg(arg!(--triggers <paths>).required(false).action(ArgAction::Append))
        .get_matches();

    let triggers = matches
        .get_many::<String>("triggers")
        .unwrap_or_default()
        .map(|v| v.as_str())
        .collect::<Vec<_>>();
    let version_file = matches
        .get_one::<String>("version_file")
        .expect("provide a version file path. Use option --help for more info.");
    // println!("triggers: {:?}", &triggers);
    let path = Path::new(version_file);

    if check_triggers(version_file.as_ref(), &triggers) {
        update_version_file(&path);
    }
}

/// see if one of the trigger files is more recent than the version file.
fn check_triggers(version_file: &Path, triggers: &Vec<&str>) -> bool {
    let version_time = get_file_time(version_file);
    triggers.iter()
        .map(|path| get_file_time(&Path::new(path)))
        .find(|time| time > &version_time)
        .is_some()
}

fn get_file_time(path: &Path) -> SystemTime {
    let meta_result = fs::metadata(path);
    let Ok(meta) = meta_result else {
        println!("{}", meta_result.err().unwrap());
        panic!("full path: {}", path.absolutize().unwrap().to_str().unwrap().to_string());
    };
    meta.modified().unwrap()
}

fn update_version_file(path: &Path) {
    let full_path = path.absolutize().unwrap().to_str().unwrap().to_string();
    let read_result = fs::read_to_string(path);
    let Ok(content) = read_result else {
        //try to make the file?
        println!("{}", read_result.err().unwrap());
        println!("{}", full_path);
        return;
    };
    let parts: Vec<_> = content.split(".").collect();
    if parts.len() != 3 {
        println!("version file does not contain a value in the format 1.2.3");
        return;
    }
    let Ok(major) = parts[0].parse::<i32>() else {
        println!("major version number is not an int: `{}`", parts[0]);
        return;
    };
    let Ok(minor) = parts[1].parse::<i32>() else {
        println!("minor version number is not an int: `{}`", parts[1]);
        return;
    };
    let Ok(mut build) = parts[2].parse::<i32>() else {
        println!("build version number is not an int: `{}`", parts[2]);
        return;
    };
    build += 1;
    let version_str = format!("{}.{}.{}", major, minor, build);
    fs::write(path, version_str).expect("Could not write version file.");
}
