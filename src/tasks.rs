#![allow(clippy::needless_return)]
use chrono::{DateTime, Utc};
use crossterm::{
    cursor::{DisableBlinking, EnableBlinking, Hide, MoveTo, Show},
    execute,
    style::Print,
    terminal::{Clear, ClearType},
};
use hashbrown::HashMap;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use std::fs::File;
use std::io::prelude::*;
use std::io::stdout;
use std::path::PathBuf;

use crate::config::Config;
use crate::date_format;
use crate::print;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Task {
    pub item: String,
    pub checked: bool,
    pub board_name: String,
    pub note: bool,
    #[serde(with = "date_format")]
    pub date: DateTime<Utc>,
}

impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        self.item == other.item
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Data {
    pub tasks: Vec<Task>,
}

const COMMAND: usize = 0;
const ARGUMENT: usize = 1;

#[allow(dead_code)]
fn clear() {
    execute!(
        stdout(),
        Hide,
        DisableBlinking,
        MoveTo(0, 0),
        Clear(ClearType::All)
    )
    .ok();
}

pub fn get_tasks() -> Data {
    let mut file = match File::open(&Config::current()) {
        Err(why) => panic!("couldn't open {}: ", why),
        Ok(file) => file,
    };
    let mut contents = String::new();
    let mut data = Data { tasks: Vec::new() };

    file.read_to_string(&mut contents).unwrap();
    if !contents.is_empty() {
        data = toml::from_str(&contents).unwrap();
    } else {
        return data;
    }

    data
}

pub fn write_toml(file_name: PathBuf, data: &Data) {
    let mut file = File::create(file_name).unwrap();
    let output = toml::to_string(&data).unwrap();
    file.write_all(output.as_bytes()).unwrap();
}

fn append_toml(file_name: PathBuf, data: &Data) {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .append(true)
        .open(file_name)
        .unwrap();

    let output = toml::to_string(&data).unwrap();
    file.write_all(output.as_bytes()).unwrap();
}

//todo update
fn get_id(id: &mut Vec<usize>, args: Vec<String>) -> bool {
    //if use input is like: "1 - 3"
    if args.len() == 3 && args[COMMAND + 1] == *"-" {
        //check for numbers
        if args[0].parse::<usize>().is_ok() && args[2].parse::<usize>().is_ok() {
            let (x, y) = (
                args[0].parse::<usize>().unwrap(),
                args[2].parse::<usize>().unwrap(),
            );
            for i in x..y + 1 {
                id.push(i - 1);
            }
        }
    } else {
        for elem in args.iter() {
            if elem.parse::<usize>().is_ok() {
                let temp: usize = elem.parse().unwrap();
                id.push(temp - 1);
            } else {
                //negative numbers are apparently not numbers
                println!("'{}' is not a valid number!", elem);
                return false;
            }
        }
    }
    return true;
}

pub fn check_task(numbers: Vec<usize>, range: bool) -> bool {
    let mut data = get_tasks();
    if range {
        let (start, end) = (*numbers.first().unwrap(), *numbers.last().unwrap());

        if start == 0 {
            println!("'0' is not a task!");
        }

        for id in start..end + 1 {
            if id > data.tasks.len() {
                println!("'{}' is not a task!", id);
                return true;
            } else {
                data.tasks[id - 1].checked = !data.tasks[id - 1].checked;
            }
        }
    } else {
        for id in numbers {
            if id > data.tasks.len() {
                println!("'{}' is not a task!", id);
                return true;
            } else {
                data.tasks[id - 1].checked = !data.tasks[id - 1].checked;
            }
        }
    }

    write_toml(Config::current(), &data);

    return false;
}

pub fn add_task(mut args: Vec<String>) {
    //remove the command
    if args[COMMAND] == *"a" {
        args.remove(0);
    }

    let arguments: String;
    let mut name: String = "Tasks".to_string();

    //Get the board_name and task data
    if args[COMMAND].contains('!') {
        name = args[COMMAND].clone().replace('!', "");
        arguments = args[ARGUMENT..].join(" ");
    } else {
        arguments = args[COMMAND..].join(" ");
    }
    let now: DateTime<Utc> = Utc::now();
    let task = Task {
        item: arguments,
        checked: false,
        board_name: name,
        note: false,
        date: now,
    };

    let data = Data { tasks: vec![task] };
    append_toml(Config::current(), &data);
}

pub fn delete_task(mut args: Vec<String>) -> bool {
    let mut id: Vec<usize> = Vec::new();
    if args[COMMAND] == *"d" {
        args.remove(0);
    }

    if !get_id(&mut id, args) {
        return true;
    }

    let mut data = get_tasks();

    //since we're deleting tasks the size will change
    let size = data.tasks.len();
    //this is annoying but again the size chagnes
    let mut indexes_removed = 0;

    for i in id {
        if i < size {
            data.tasks.remove(i - indexes_removed);
            indexes_removed += 1;
        } else if i != 0 {
            println!("'{}' is not a task!", i + 1);
            return true;
        }
    }

    if data.tasks.is_empty() {
        File::create(Config::current()).unwrap();
        return true;
    }

    write_toml(Config::current(), &data);

    return false;
}

pub fn clear_tasks() {
    let mut data_to_append: Data = Data { tasks: Vec::new() };

    //Get finished tasks and put them in buffer
    let mut data = get_tasks();
    let mut indexes_removed = 0;

    //return if there are no tasks to clear
    if data.tasks.is_empty() {
        return;
    }

    //Copy checked tasks to new file
    for i in 0..data.tasks.len() {
        if data.tasks[i - indexes_removed].checked {
            data_to_append
                .tasks
                .push(data.tasks[i - indexes_removed].clone());
            data.tasks.remove(i - indexes_removed);
            indexes_removed += 1;
        }
    }

    if data.tasks.is_empty() {
        File::create(Config::current()).unwrap();
    } else {
        write_toml(Config::current(), &data);
    }

    append_toml(Config::old(), &data_to_append);
}

pub fn tasks() {
    let data = get_tasks();

    if data.tasks.is_empty() {
        print::help_message();
        return;
    }

    let mut board_completed: HashMap<&str, usize> = HashMap::new();
    let mut board_total: HashMap<&str, usize> = HashMap::new();
    let mut board_list: Vec<&str> = Vec::new();

    let mut tasks_total = data.tasks.len();
    let mut tasks_completed = 0;
    let now: DateTime<Utc> = Utc::now();

    //Get a list of all boards
    for elem in data.tasks.iter() {
        board_list.push(elem.board_name.as_str());
        if elem.checked && !elem.note {
            tasks_completed += 1;
        }
    }

    //Remove repeated elements
    board_list = board_list.into_iter().unique().collect();

    //Get total and completed tasks for each board
    for board in &board_list {
        //boards completed and board total
        let (mut bc, mut bt) = (0, 0);

        for elem in data.tasks.iter() {
            if elem.board_name == *board {
                bt += 1;
                if elem.checked {
                    bc += 1;
                }
            }
        }

        //push the name and value into a hashmap
        board_completed.insert(board, bc);
        board_total.insert(board, bt);
    }

    //Remove the default board, we will print this last
    board_list.retain(|&x| x != "Tasks");

    let mut notes_total = 0;
    let mut index = 0;

    clear();

    //Print all the custom boards
    for board in board_list {
        print::header(board_completed[board], board_total[board], board);
        for elem in data.tasks.iter() {
            let day = (now - elem.date).num_days();
            if elem.board_name == board {
                index += 1;
                if elem.note {
                    print::note(index, elem.item.as_str(), tasks_total);
                    notes_total += 1;
                } else {
                    print::task(index, elem.checked, elem.item.as_str(), day, tasks_total);
                }
            }
        }
        println!();
    }

    //Print the header for the default board
    print::header(board_completed["Tasks"], board_total["Tasks"], "Tasks");

    //Print the default board
    for elem in data.tasks.iter() {
        if elem.board_name == "Tasks" {
            index += 1;
            let day = (now - elem.date).num_days();
            if elem.note {
                print::note(index, elem.item.as_str(), tasks_total);
                notes_total += 1;
            } else {
                print::task(
                    index,
                    elem.checked,
                    elem.item.as_str(),
                    day,
                    board_total["Tasks"],
                );
            }
        }
    }

    println!();

    //Don't count the notes
    tasks_total -= notes_total;

    print::footer(tasks_completed, tasks_total, notes_total);

    execute!(stdout(), Print("\n"), Show, EnableBlinking).unwrap();
}

pub fn old_tasks() {
    let mut file = match File::open(&Config::old()) {
        Err(why) => panic!("couldn't open {}: ", why),
        Ok(file) => file,
    };

    let mut contents = String::new();

    file.read_to_string(&mut contents).unwrap();

    let now: DateTime<Utc> = Utc::now();
    if !contents.is_empty() {
        let data: Data = toml::from_str(&contents).unwrap();
        let total_tasks = data.tasks.len();
        //how long ago the task was added in days
        for i in 0..data.tasks.len() {
            let day = (now - data.tasks[i].date).num_days();
            print::task(
                i + 1,
                data.tasks[i].checked,
                &data.tasks[i].item,
                day,
                total_tasks,
            );
        }
    } else {
        println!("Task archive is empty.");
    }
}

pub fn add_note(args: Vec<String>) {
    let arguments = args[ARGUMENT..].join(" ");
    let now: DateTime<Utc> = Utc::now();

    let task = Task {
        item: arguments,
        checked: false,
        board_name: "Tasks".to_string(),
        note: true,
        date: now,
    };

    let data = Data { tasks: vec![task] };
    append_toml(Config::current(), &data);
}
