//! # My library crate which consists of helper functions used either in the main libary or handler library
//!
//! ## Features
//!
//! - **run_commands_line:** when given a list of commands which are in line, runs them and gets the final output of that line
//! - **my_wait:** blocks/waits for when a thread is availble for use
//! - **my_notify:** notifies when a thread is avaiable
//! - **get_concurrent_jobs_j_tag:** gets the concurrent jobs argument and returns a usize if is -J tag
//! - **get_concurrent_jobs_and_remote_r_tag:** gets the concurrent jobs argument and returns a usize if is -r tag
//! - **get_termination_control:** gets the termination control from the given args by the input

use std::process::Output;
use std::sync::atomic::AtomicBool;
use std::sync::Condvar;
use std::sync::Mutex;
use std::{
    process::exit, process::Command, sync::atomic::Ordering,
    sync::Arc
};

/// Given the arguments discussed below, runst he command given as a vector of vector of String (e.g. [["echo", hi"], ["echo", "bye"]])
///
/// # Arguments
///
/// * `commands_in_line` - vector of vector of strings which represents the commands in line
/// * `command_invalid` - a flag which changes if the command line returns invalid
/// * `termination_control` - the termination control represented by a string
pub fn run_commands_line(
    commands_in_line: Vec<Vec<String>>,
    command_invalid: Arc<AtomicBool>,
    termination_control: String,
) -> (String, bool) {
    let mut output_val: String = "".to_string();
    let mut command_fail = false;
    let mut output: Result<Output, _>;

    for command in commands_in_line {
        if let Some(first_element) = command.get(0) {
            let mut first_elem_command = Command::new(first_element);
            first_elem_command.args(&command[1..]);

            output = first_elem_command.output();

            match output {
                Ok(ret) => {
                    if ret.status.success() {
                        let result = String::from_utf8_lossy(&ret.stdout);
                        if command_invalid.load(Ordering::Relaxed) && termination_control == "eager"
                        {
                            break;
                        } else {
                            output_val = output_val + &result;
                        }
                    } else {
                        command_fail = true;
                        break;
                    }
                }
                Err(_) => {
                    command_fail = true;
                    println!("Failed to spawn command!");
                    break;
                }
            }
        }
    }
    (output_val.to_string(), command_fail)
}

/// Given the arguments discussed below, waits/locks until a thread opens up for use
///
/// # Arguments
///
/// * `thread_count` - an Arc tuple which consists of the count of threads and Conditional var
/// * `command_invalid` - a flag which changes if the command line returns invalid
/// * `termination_control` - the termination control represented by a string
/// * `concurrent_jobs` - a usize which represents the number concurrent jobs that can be used
pub fn my_wait(thread_count: Arc<(Mutex<usize>, Condvar)>, concurrent_jobs: usize) {
    let (lock, cvar) = &*thread_count;
    let mut new_thread_count = lock.lock().unwrap();

    *new_thread_count += 1;

    // until a thread is freed up
    while *new_thread_count > concurrent_jobs {
        new_thread_count = cvar.wait(new_thread_count).unwrap();
    }
}

/// Given the arguments discussed below, notifies all the threads and subtracts a new thread indicating it is being used
///
/// # Arguments
///
/// * `thread_count` - an Arc tuple which consists of the count of threads and Conditional var
pub fn my_notify(thread_count: Arc<(Mutex<usize>, Condvar)>) {
    let (lock, cvar) = &*thread_count;
    let mut new_thread_count = lock.lock().unwrap();
    if let Some(result) = new_thread_count.checked_sub(1) {
        *new_thread_count = result;
    }
    // notify all a thread has finished up and a thread is available for use
    cvar.notify_all();
}

/// Given the arguments discussed below, an edge case handled which extracts
/// the concurrent jobs (returns as a usize) when the user inputs an arguments with -J tags
///
/// # Arguments
///
/// * `args` - the args inputted by the user
/// * `idx` - the idx at which the extraction happens
///
/// # Example
/// ```
/// # use helpers::get_concurrent_jobs_j_tag;
/// let args: Vec<String> = vec!["cargo".to_string(), "run".to_string(), "--".to_string(), "-J".to_string(), "2".to_string()];
/// let ret: usize = get_concurrent_jobs_j_tag(args, 4);
///
/// assert_eq!(ret, 2);
/// ```

pub fn get_concurrent_jobs_j_tag(args: Vec<String>, idx: usize) -> usize {
    let concurrent_jobs_str = args.get(idx).unwrap();
    match concurrent_jobs_str.parse::<usize>() {
        Ok(number) => number,
        Err(err) => {
            eprintln!("Must be int: {}", err);
            exit(1);
        }
    }
}
/// Given the arguments discussed below, an edge case handled which extracts
/// the concurrent jobs and remote (returns as a tuple) when the user inputs an arguments with -r tags
///
/// # Arguments
///
/// * `args` - the args inputted by the user
/// * `idx` - the idx at which the extraction happens
///
/// # Example
/// ```
/// # use helpers::get_concurrent_jobs_and_remote_r_tag;
/// let args: Vec<String> = vec!["cargo".to_string(), "run".to_string(), "--".to_string(), "-r".to_string(), "localhost:12345/2".to_string()];
/// let ret = get_concurrent_jobs_and_remote_r_tag(args, 4);
///
/// assert_eq!(ret, ("localhost".to_string(), 12345 as i16, 2 as usize));
/// ```
pub fn get_concurrent_jobs_and_remote_r_tag(args: Vec<String>, idx: usize) -> (String, i16, usize) {
    let port_num;
    let concurrent_jobs;
    let address;

    let connect_to = args.get(idx).unwrap();
    let three_parts: Vec<&str> = connect_to.split(':').collect();
    if let Some(port_num_and_threads) = three_parts.get(1) {
        let port_parts: Vec<&str> = port_num_and_threads.split('/').collect();
        if let Some(port_number_str) = port_parts.first() {
            match port_number_str.parse::<i16>() {
                Ok(number) => {
                    port_num = number;
                }
                Err(_) => {
                    eprintln!("Port must be int");
                    exit(1);
                }
            }
        } else {
            eprintln!("Port must be provided");
            exit(1);
        }
        if let Some(concurrent_jobs_str) = port_parts.get(1) {
            match concurrent_jobs_str.parse::<usize>() {
                Ok(number) => {
                    concurrent_jobs = number;
                }
                Err(_) => {
                    eprintln!("Concurrent job must be int");
                    exit(1);
                }
            }
        } else {
            eprintln!("Concurrent jobs must be provided");
            exit(1);
        }
    } else {
        eprintln!("Invalid args");
        exit(1);
    }

    if let Some(address_str) = three_parts.first() {
        address = address_str;
    } else {
        eprintln!("Invalid args");
        exit(1);
    }

    (address.to_string(), port_num, concurrent_jobs)
}

/// Given the arguments discussed below, an edge case handled which extracts
/// the termination control (returns as a String)
///
/// # Arguments
///
/// * `args` - the args inputted by the user
/// * `idx` - the idx at which the extraction happens
///
/// # Example
/// ```
/// # use helpers::get_termination_control;
/// let args: Vec<String> = vec!["cargo".to_string(), "run".to_string(), "--".to_string(), "-J".to_string(), "2".to_string(), "--halt".to_string(), "lazy".to_string()];
/// let ret = get_termination_control(args, 6);
///
/// assert_eq!(ret, "lazy");
/// ```
pub fn get_termination_control(args: Vec<String>, idx: usize) -> String {
    match args.get(idx) {
        Some(fifth_arg) => {
            if fifth_arg == "lazy" {
                "lazy".to_string()
            } else if fifth_arg == "eager" {
                "eager".to_string()
            } else if fifth_arg == "never" {
                "never".to_string()
            } else {
                eprintln!("Not valid arg");
                exit(1);
            }
        }
        None => {
            eprintln!("Not valid number of args");
            exit(1);
        }
    }
}