use std::io::Write;

use chrono::{TimeZone, Utc};
use database::Database;
use regex::Regex;

mod database;
mod ui;

pub struct App {
    pub db: Database,
    args: Vec<String>,
}

impl App {
    pub fn run() {
        let app = Self {
            db: Database::default(),
            args: std::env::args().skip(1).collect(),
        };

        match app.args.len() {
            0 => app.print(),
            _ => {
                match app.args[0].as_str() {
                    "n" | "d" if app.args.len() == 1 => {
                        return ui::missing_args(app.args[0].as_str())
                    }
                    "h" | "help" => return ui::help(),
                    "v" | "version" => return println!("t {}", env!("CARGO_PKG_VERSION")),
                    "o" | "old" => return app.print_old(),
                    "n" => {
                        if let Err(err) = app.add(true) {
                            return println!("{}", err);
                        }
                    }
                    "d" => match app.ids() {
                        Ok(ids) => app.db.delete_tasks(&ids),
                        Err(err) => return println!("{}", err.unwrap_or("")),
                    },
                    "cls" => app.db.clear_tasks(),
                    _ => match app.ids() {
                        Ok(ids) => app.db.check_tasks(&ids),
                        //error with numbers or task?
                        Err(err) => match err {
                            Some(err) => return println!("{}", err),
                            None => {
                                //check for for input errors
                                if let Err(err) = app.add(false) {
                                    return println!("{}", err);
                                }
                            }
                        },
                    },
                }

                app.print();
            }
        }
    }
    pub fn print(&self) {
        let total_tasks = self.db.total_tasks();
        let total_notes = self.db.total_notes();
        let total = self.db.total();

        if total == 0 {
            return ui::help_message();
        }

        let total_checked = self.db.total_checked();
        let boards = self.db.get_boards();
        let mut i = 1;

        ui::clear();

        for board in boards {
            ui::header(board.checked, board.total, &board.name);

            for task in board.tasks {
                if task.note {
                    ui::note(i, &task.content, total);
                } else {
                    let date = Utc
                        .datetime_from_str(&task.date, "%Y-%m-%d %H:%M:%S")
                        .unwrap();
                    let days = (Utc::now() - date).num_days();
                    ui::task(i, task.checked, &task.content, days, total);
                }
                i += 1;
            }
            ui::new_line();
        }

        ui::footer(total_checked, total_tasks, total_notes);
    }
    fn print_old(&self) {
        let old_tasks = self.db.get_old();
        if old_tasks.is_empty() {
            return println!("No old tasks.");
        }
        ui::clear();

        ui::old_header();

        for (i, task) in old_tasks.iter().enumerate() {
            ui::note(i + 1, task, old_tasks.len());
        }
    }
    fn ids(&self) -> Result<Vec<usize>, Option<&str>> {
        let args = if self.args.iter().any(|arg| arg == &String::from('d')) {
            &self.args[1..]
        } else {
            &self.args[0..]
        }
        .join(" ")
        .trim()
        .to_string();

        let nums = Regex::new("^[0-9 ]*$").unwrap();
        let range = Regex::new(r"^(?x)(?P<first>\d+)(\s+)?-(\s+)?(?P<last>\d+)$").unwrap();

        if nums.captures(&args).is_some() {
            args.split(' ')
                .map(|str| {
                    if let Ok(num) = str.parse() {
                        if num > self.db.total() {
                            Err(Some("Task does not exist."))
                        } else {
                            Ok(num)
                        }
                    } else {
                        Err(Some("Invalid number."))
                    }
                })
                .collect()
        } else if let Some(caps) = range.captures(&args) {
            let first: usize = caps["first"].parse().unwrap();
            let last: usize = caps["last"].parse().unwrap();

            if first > last {
                return Err(Some(
                    "Invalid range! First number must be smaller than last.",
                ));
            }

            Ok((first..last + 1).collect())
        } else {
            Err(None)
        }
    }
    fn add(&self, is_note: bool) -> Result<(), &str> {
        let args = if is_note { &self.args[1..] } else { &self.args };
        let mut board_name = None;

        let item = if args[0].contains('!') {
            if args.len() >= 2 {
                //t !Task 'sample task'
                board_name = Some(args[0].replace('!', ""));
                args[1..].join(" ")
            } else {
                let input: Vec<&str> = args[0].split(' ').collect();

                if input.len() == 1 {
                    //t '!Tasks'
                    return Err("Missing task!");
                } else {
                    //t '!Tasks sample task'
                    board_name = Some(input[0].replace('!', ""));
                    input[1..].join(" ")
                }
            }
        } else {
            //t 'sample task'
            args[0..].join(" ")
        };

        if is_note {
            self.db.insert_note(&item, board_name);
        } else {
            self.db.insert_task(&item, board_name);
        }
        Ok(())
    }
}

impl Drop for App {
    fn drop(&mut self) {
        std::io::stdout().flush().unwrap();
    }
}

fn main() {
    App::run();
}
