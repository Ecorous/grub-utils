// Copyright 2023 ecorous
// 
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
// 
//     http://www.apache.org/licenses/LICENSE-2.0
// 
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use clap::{Parser, Subcommand};
use std::process::Command;

#[derive(Parser, Debug)]
struct GrubUtils {
    #[command(subcommand)]
    subcommand: GCommand,
}

fn help(arg: &str) -> String {
    let RED = "\x1b[31";
    let RESET = "\x1b[0m";
    let editor = format!("The edtor to use. {RED}WARNING: the editor you specify will be run as root, meaning it is granted full access to your system.{RESET}\nPriority for detecting editor: This argument <- EDITOR variable <- default of /usr/bin/nano");
    match arg {
        "no_generate" => String::from("Do NOT generate a grub configuration after editing"),
        "file" => String::from("The file to edit. Note that this does NOT affect generation.\ndefault: /etc/default/grub"),
        "output" => String::from("The path to output the generated config to\ndefault: /boot/grub/grub.cfg"),
        "editor" => editor,
        _ => String::from("Unknown help!")
    }
}

#[derive(Subcommand, Debug)]
enum GCommand {
    Edit {
        #[arg(long, help = help("no_generate"))]
        no_generate: bool,
        #[arg(long, short = 'f', help = help("file"))]
        file: Option<String>,
        #[arg(long, short = 'o', help = help("output"))]
        output: Option<String>,
        #[arg(long, short = 'e', help = help("editor"))]
        editor: Option<String>,
    },
    Generate {
        #[arg(long, short = 'o')]
        output: Option<String>,
    },
}

fn get_editor(arg: Option<String>) -> String {
    let mut _out: String;
    let env = std::env::var("EDITOR").unwrap_or(String::from(""));
    if let Some(argument) = arg {
        argument
    } else if env.as_str() != "" {
        env
    } else {
        String::from("/usr/bin/nano")
    }
}

fn generate_grub(output: String) {
    let command_status = Command::new("grub-mkconfig")
        .arg("-o")
        .arg(output)
        .status()
        .unwrap_or_else(|e| {
            eprintln!("Failed to execute command: {}", e);
            std::process::exit(1);
        });
    println!(
        "grub-mkconfig exited with code {}",
        command_status.code().unwrap_or(-127)
    )
}

fn main() {
    if !is_root() {
        println!("Warning: GrubUtils was not run using root privileges, attempting to use sudo to elevate privileges");
        let mut args = std::env::args();
        let program = args.next().unwrap();
        let status = Command::new("sudo")
            .arg("-E")
            .arg(&program)
            .args(args)
            .status()
            .unwrap_or_else(|e| {
                eprintln!("Failed to execute command: {}", e);
                std::process::exit(1);
            });
        std::process::exit(status.code().unwrap_or(1));
    }
    println!("GrubUtils is running as root");
    let cli = GrubUtils::parse();
    match cli.subcommand {
        GCommand::Edit {
            no_generate,
            file,
            output,
            editor,
        } => {
            let path: String;
            if let Some(x) = file {
                path = x;
            } else {
                path = String::from("/etc/default/grub")
            }
            println!("Using grub file: {}", path);
            let true_editor = get_editor(editor);
            println!("Using editor: {}", true_editor);
            let editor_output = Command::new(true_editor.clone())
                .arg(path)
                .status()
                .unwrap_or_else(|e| {
                    eprintln!("Failed to execute command: {}", e);
                    std::process::exit(1);
                });
            println!(
                "Editor exited with code {}",
                editor_output.code().unwrap_or(-127)
            );
            if !no_generate {
                let mut outfile: String;
                if let Some(argout) = output {
                    outfile = argout
                } else {
                    outfile = String::from("/boot/grub/grub.cfg")
                }
                generate_grub(outfile);
            }
        }
        GCommand::Generate { output } => {
            let mut outfile: String;
            if let Some(argout) = output {
                outfile = argout
            } else {
                outfile = String::from("/boot/grub/grub.cfg")
            }
            generate_grub(outfile);
        }
    }
}

fn is_root() -> bool {
    unsafe { libc::geteuid() == 0 }
}
