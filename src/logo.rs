use colored::Colorize;

use crate::config::Environment;

pub const LOGO: &str = r"
#####################################

           ／＞　　フ
　　　 　　|  _　 _ l
　 　　 　／ヽ ミ＿xノ
　　 　 /　　　 　 |
　　　 /　 ヽ　　 ﾉ
　 　 │　　|　|　|
　／￣|　　 |　|　|
　| (￣ヽ＿_ヽ_)__)
　＼二つ

,-.,-.,-. .--.  .--. .-..-..-.
: ,. ,. :' '_.'' .; :: `; `; :
:_;:_;:_;`.__.'`.__.'`.__.__.'

#####################################
";

pub fn print_logo(environment: Environment, configs: crate::config::Config) {
    println!("{}", LOGO.bright_red());

    println!("environment: {}", environment.as_str().bright_magenta());

    let mut worker_line = Vec::new();

    worker_line.push(format!(
        "listening on {}:{}",
        configs.server.host.green(),
        configs.server.port.to_string().bright_magenta()
    ));

    println!();
    println!("{}", worker_line.join("\n"));
}
