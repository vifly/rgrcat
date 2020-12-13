use std::env;
use std::io;
use std::process::exit;
use std::fs::File;
use std::path::PathBuf;
use std::io::{BufReader, BufRead};
use std::collections::HashMap;
use regex::Regex;
use std::ffi::OsString;


struct ColourConfig {
    regexp: String,
    colours: Vec<String>,
    count: String,
    command: String,
    skip: String,
    replace: String,
    concat: String,
}


fn get_config_name_from_args() -> String {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("You are not supposed to call rgrcat directly, but the usage is: rgrcat conffile");
        exit(-1);
    }
    args[1].clone()
}


fn get_env_var(key: &str, default: &str) -> String {
    match env::var(key) {
        Ok(val) => val,
        Err(_e) => default.to_string()
    }
}


fn get_config_path(config_name: &String) -> Option<String> {
    let home = env::var("HOME").expect("Can not find HOME path!");
    let xdg_config = get_env_var("XDG_CONFIG_HOME", "");
    let xdg_data = get_env_var("XDG_CONFIG_HOME", "");
    let home_path = PathBuf::from(&home);
    let mut xdg_config_path = PathBuf::from(&xdg_config);
    let mut xdg_data_path = PathBuf::from(&xdg_data);
    let mut config_dir: Vec<PathBuf> = vec![];

    if xdg_config_path.eq(&OsString::from("")) {
        xdg_config_path = home_path.join(".config/grc");
    }
    if xdg_data_path.eq(&OsString::from("")) {
        xdg_data_path = home_path.join(".local/share/grc");
    }

    config_dir.push(xdg_config_path);
    config_dir.push(xdg_data_path);
    config_dir.push(home_path.join(".grc"));
    config_dir.push(PathBuf::from("/usr/local/share/grc"));
    config_dir.push(PathBuf::from("/usr/share/grc"));

    for dir in config_dir {
        let config_file_path = dir.join(config_name);
        if config_file_path.exists() && !config_file_path.is_dir() {
            return Some(String::from(config_file_path.to_str().unwrap()));
        }
    }
    eprintln!("config file [{}] not found", config_name);
    return None;
}


fn is_config_split_line(line: &String) -> bool {
    // It's a comment line.
    if line.starts_with('#') {
        false
    // It's a blank line.
    } else if line.eq(&"".to_string()) {
        false
    // First char not in ascii alphabet, so it's a split line.
    } else if !line.chars().next().unwrap().is_ascii_alphabetic() {
        true
    } else {
        false
    }
}


fn parse_config_line(line: &String) -> Option<(String, String)> {
    if line.starts_with('#') {
        None
    } else if line.eq(&"".to_string()) {
        None
    } else {
        let key_val: Vec<&str> = line.splitn(2, "=").collect();

        if key_val.len() != 2 {
            eprintln!("Error in configuration, I expect keyword=value line");
            eprintln!("But I got instead: {}", line);
            return None;
        }

        let mut key = "";
        let value = key_val[1];
        if key_val[0].starts_with("colo") {
            key = "colours";
        } else {
            key = key_val[0];
        }

        Some((key.to_string(), value.to_string()))
    }
}


fn parse_config(path: &String) -> Result<Vec<ColourConfig>, io::Error> {
    // Ref: https://riptutorial.com/rust/example/4275/read-a-file-line-by-line
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut key_val_list: Vec<(String, String)> = vec![];
    let mut config_list: Vec<ColourConfig> = vec![];
    for (_index, line) in reader.lines().enumerate() {
        let line = line?;
        if is_config_split_line(&line) {
            let mut config = ColourConfig::new();
            config.insert_content(&key_val_list);
            config_list.push(config);
            key_val_list.clear();
        } else {
            match parse_config_line(&line) {
                None => continue,
                Some(key_val) => key_val_list.push(key_val)
            };
        }
    }

    let mut config = ColourConfig::new();
    config.insert_content(&key_val_list);
    config_list.push(config);

    Ok(config_list)
}


fn get_colour(colour_name: &str) -> String {
    // Use \x1b instead of \033. Ref: https://stackoverflow.com/questions/33139248/i-cannot-print-color-escape-codes-to-the-terminal
    let mut colour_map: HashMap<&str, &str> = HashMap::new();
    colour_map.insert("none", "");
    colour_map.insert("default", "\x1b[0m");
    colour_map.insert("bold", "\x1b[1m");
    colour_map.insert("underline", "\x1b[4m");
    colour_map.insert("blink", "\x1b[5m");
    colour_map.insert("reverse", "\x1b[7m");
    colour_map.insert("concealed", "\x1b[8m");

    colour_map.insert("black", "\x1b[30m");
    colour_map.insert("red", "\x1b[31m");
    colour_map.insert("green", "\x1b[32m");
    colour_map.insert("yellow", "\x1b[33m");
    colour_map.insert("blue", "\x1b[34m");
    colour_map.insert("magenta", "\x1b[35m");
    colour_map.insert("cyan", "\x1b[36m");
    colour_map.insert("white", "\x1b[37m");

    colour_map.insert("on_black", "\x1b[40m");
    colour_map.insert("on_red", "\x1b[41m");
    colour_map.insert("on_green", "\x1b[42m");
    colour_map.insert("on_yellow", "\x1b[43m");
    colour_map.insert("on_blue", "\x1b[44m");
    colour_map.insert("on_magenta", "\x1b[45m");
    colour_map.insert("on_cyan", "\x1b[46m");
    colour_map.insert("on_white", "\x1b[47m");

    colour_map.insert("beep", "\007");
    colour_map.insert("previous", "prev");
    colour_map.insert("unchanged", "unchanged");

    // non-standard attributes, supported by some terminals
    colour_map.insert("dark", "\x1b[2m");
    colour_map.insert("italic", "\x1b[3m");
    colour_map.insert("rapidblink", "\x1b[6m");
    colour_map.insert("strikethrough", "\x1b[9m");

    // aixterm bright color codes
    // prefixed with standard ANSI codes for graceful failure
    colour_map.insert("bright_black", "\x1b[30;90m");
    colour_map.insert("bright_red", "\x1b[31;91m");
    colour_map.insert("bright_green", "\x1b[32;92m");
    colour_map.insert("bright_yellow", "\x1b[33;93m");
    colour_map.insert("bright_blue", "\x1b[34;94m");
    colour_map.insert("bright_magenta", "\x1b[35;95m");
    colour_map.insert("bright_cyan", "\x1b[36;96m");
    colour_map.insert("bright_white", "\x1b[37;97m");

    colour_map.insert("on_bright_black", "\x1b[40;100m");
    colour_map.insert("on_bright_red", "\x1b[41;101m");
    colour_map.insert("on_bright_green", "\x1b[42;102m");
    colour_map.insert("on_bright_yellow", "\x1b[43;103m");
    colour_map.insert("on_bright_blue", "\x1b[44;104m");
    colour_map.insert("on_bright_magenta", "\x1b[45;105m");
    colour_map.insert("on_bright_cyan", "\x1b[46;106m");
    colour_map.insert("on_bright_white", "\x1b[47;107m");

    // We don't raise Exception like original grc, instead of return default value.
    let colour = match colour_map.get(colour_name) {
        Some(val) => val,
        None => "\x1b[0m"
    };

    colour.to_string()
}


fn get_colour_list(raw_colour: &String) -> Vec<String> {
    let mut colour_list = vec![];
    let colour_group: Vec<&str> = raw_colour.split(',').collect();
    for colours in colour_group {
        let colour_group: Vec<&str> = colours.split(' ').collect();
        for colour in colour_group {
            if colour.ne("") {
                colour_list.push(get_colour(colour));
            }
        }
    }

    colour_list
}


impl ColourConfig {
    fn new() -> ColourConfig {
        ColourConfig {
            regexp: String::new(),
            colours: vec![String::new()],
            count: "more".to_string(),
            command: String::new(),
            skip: String::new(),
            replace: String::new(),
            concat: String::new(),
        }
    }


    fn insert_content(&mut self, content: &Vec<(String, String)>) {
        for item in content {
            if item.0.eq("regexp") {
                self.regexp = item.1.clone();
            } else if item.0.eq("colours") {
                self.colours = get_colour_list(&item.1);
            } else if item.0.eq("count") {
                self.count = item.1.clone();
            } else if item.0.eq("command") {
                self.command = item.1.clone();
            } else if item.0.eq("skip") {
                self.skip = item.1.clone();
            } else if item.0.eq("replace") {
                self.replace = item.1.clone();
            } else if item.0.eq("concat") {
                self.concat = item.1.clone();
            } else {
                eprintln!("{} is not key", item.0);
            }
        }
    }
}


fn get_colour_str(content: &str, colour: &String) -> String {
    let mut result = colour.clone();
    result.push_str(&content);
    // Make sure string after result use default colour.
    result.push_str(&get_colour("default"));
    result
}


fn get_colour_line_by_re(line: &str, colour: &str, re: &Regex) -> String {
    let mut result = line.clone().to_string();
    for m in re.find_iter(line) {
        let match_str = String::from(&line[m.start()..m.end()]);
        let colour_str = get_colour_str(&match_str, &colour.to_string());
        result = line.replace(&match_str, &colour_str);
    }

    result
}


fn get_output_line_by_config(line: &str, config_list: &Vec<ColourConfig>) -> String {
    let mut result = line.clone().to_string();
    for config in config_list {
        if config.count.eq("block") {
            get_colour_str(line, &config.colours[0]);
        } else if config.count.eq("unblock") {
            get_colour_str(line, &get_colour("default"));
        } else {
            if !&config.colours.contains(&"unchanged".to_string()) {
                let re = Regex::new(&config.regexp[..]).unwrap();
                // Todo config.colours[0] is temp
                result = get_colour_line_by_re(&result, &config.colours[0], &re);
            }
        }
    }

    result
}


fn is_skip_input_line(config_list: &Vec<ColourConfig>) -> bool {
    for config in config_list {
        if config.skip.eq("yes") || config.skip.eq("1") || config.skip.eq("true") {
            return true;
        }
    }
    false
}


fn process_stdio(config_list: &Vec<ColourConfig>) {
    // Ref: https://doc.rust-lang.org/std/io/struct.Stdin.html#method.read_line
    let mut input = String::new();

    loop {
        match io::stdin().read_line(&mut input) {
            Ok(n) => {
                if n == 0 {
                    break;
                }
                if !is_skip_input_line(config_list) {
                    let input = input.trim_end();
                    let result = get_output_line_by_config(input, config_list);
                    println!("{}", result);
                }
            }
            Err(error) => {
                eprintln!("error: {}", error);
                exit(-1);
            }
        }
        input.clear();
    }
}


fn main() {
    let config_name = get_config_name_from_args();
    let config_path = match get_config_path(&config_name) {
        Some(path) => path,
        None => exit(-1)
    };

    let config_list = parse_config(&config_path.to_string()).unwrap_or_else(|err| {
        eprintln!("Can not read {}", config_path);
        eprintln!("{}", err);
        exit(-1);
    });

    process_stdio(&config_list);
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_colour_list() {
        assert_eq!(get_colour_list(&"default,blink ,yellow".to_string()), vec!["\u{1b}[0m", "\u{1b}[5m", "\u{1b}[33m"]);
    }
}
