//! # My main library crate for pars
//!
//! Implementation of implementing a subset of the functionality of GNU parallel in Rust
//!
//! ## Features
//!
//! - **main:** main functionn which takes in the user arguments and determines what kind of program to
//!             run depending on the arguments given and the tags attached to it

use pars_libs::Remote;
use std::{env, process::exit};

// my crates
use handlers::{handle, run_server, RemoteDetails};
use helpers::{
    get_concurrent_jobs_and_remote_r_tag, get_concurrent_jobs_j_tag, get_termination_control,
};

fn main() {
    // get concurrent jobs
    let mut concurrent_jobs: usize = 0;
    let args: Vec<String> = env::args().collect();
    let mut port_num: i16 = -1;
    let mut address = "".to_string();
    let mut remote_connection = false;
    let mut client = false;
    let mut multiple_remotes = false;
    let number_of_remotes = &args
        .iter()
        .filter(|&x| x == "-r" || x == "--remote")
        .count();
    let mut remotes_array: Vec<RemoteDetails> = Vec::new();

    // default is Never
    let mut termination_control = "never".to_string();

    if args.len() == 3
        && (args.get(1).unwrap() == "-J"
            || args.get(1).unwrap() == "-r"
            || args.get(1).unwrap() == "--remote")
    {
        if args.get(1).unwrap() == "-J" {
            concurrent_jobs = get_concurrent_jobs_j_tag(args.clone(), 2);
        } else if args.get(1).unwrap() == "-r" || args.get(1).unwrap() == "--remote" {
            remote_connection = true;
            let remote_concurrent_jobs = get_concurrent_jobs_and_remote_r_tag(args.clone(), 2);
            address = remote_concurrent_jobs.0;
            port_num = remote_concurrent_jobs.1;
            concurrent_jobs = remote_concurrent_jobs.2;
        }
    }

    if args.len() >= 5 {
        if (args.get(1).unwrap() == "-J"
            || args.get(1).unwrap() == "-r"
            || args.get(1).unwrap() == "--remote")
            && (args.get(3).unwrap() == "-e" || args.get(3).unwrap() == "--halt")
        {
            if args.get(1).unwrap() == "-J" {
                concurrent_jobs = get_concurrent_jobs_j_tag(args.clone(), 2);
            } else if args.get(1).unwrap() == "-r" || args.get(1).unwrap() == "--remote" {
                remote_connection = true;
                let remote_concurrent_jobs = get_concurrent_jobs_and_remote_r_tag(args.clone(), 2);
                address = remote_concurrent_jobs.0;
                port_num = remote_concurrent_jobs.1;
                concurrent_jobs = remote_concurrent_jobs.2;
            }
            // get the termination control
            termination_control = get_termination_control(args.clone(), 4);
        }
        if (args.get(3).unwrap() == "-J"
            || args.get(3).unwrap() == "-r"
            || args.get(3).unwrap() == "--remote")
            && (args.get(1).unwrap() == "-e" || args.get(1).unwrap() == "--halt")
        {
            if args.get(3).unwrap() == "-J" {
                concurrent_jobs = get_concurrent_jobs_j_tag(args.clone(), 4);
            } else if args.get(3).unwrap() == "-r" || args.get(3).unwrap() == "--remote" {
                remote_connection = true;
                let remote_concurrent_jobs = get_concurrent_jobs_and_remote_r_tag(args.clone(), 4);
                address = remote_concurrent_jobs.0;
                port_num = remote_concurrent_jobs.1;
                concurrent_jobs = remote_concurrent_jobs.2;
            }

            // get the termination control
            termination_control = get_termination_control(args.clone(), 2);
        }
    }
    // check if client
    if args.len() == 6 && args.get(5).unwrap() == "--client" {
        client = true;
    }

    // check if there are more than one remotes
    if args.len() >= 7 || number_of_remotes > &1 {
        multiple_remotes = true;
        let mut idx = 1;
        while idx < args.len() - 1 {
            if args.get(idx).unwrap() == "-r" || args.get(idx).unwrap() == "--remote" {
                let remote_concurrent_jobs =
                    get_concurrent_jobs_and_remote_r_tag(args.clone(), idx + 1);
                address = remote_concurrent_jobs.0;
                port_num = remote_concurrent_jobs.1;
                concurrent_jobs = remote_concurrent_jobs.2;

                let remote_element = RemoteDetails {
                    remote: Remote {
                        addr: address.to_string(),
                        port: port_num as u16,
                    },
                    concurrent_jobs,
                    child_stdin: None,
                    child_stdout: None,
                };
                remotes_array.push(remote_element);
            }
            if args.get(idx).unwrap() == "--halt" || args.get(idx).unwrap() == "-e" {
                termination_control = get_termination_control(args.clone(), idx + 1);
            }
            idx += 1;
        }
    }

    // some more error checks
    if (args.len() != 3 && args.len() != 5 && args.len() != 6 && !multiple_remotes)
        || (remote_connection && (address == *"" || port_num < 0))
        || (concurrent_jobs == 0)
        || (multiple_remotes && remotes_array.len() < 2)
    {
        eprintln!("Invalid args");
        exit(1);
    }

    if !remote_connection && !multiple_remotes {
        // note this also handles for client. Just takes a flag which represents whether its a client or not
        handle(concurrent_jobs, termination_control, client);
    } else if !multiple_remotes {
        let remote_vec: Vec<RemoteDetails> = vec![RemoteDetails {
            remote: Remote {
                addr: address.to_string(),
                port: port_num as u16,
            },
            concurrent_jobs,
            child_stdin: None,
            child_stdout: None,
        }];
        run_server(termination_control, remote_vec);
    } else {
        run_server(termination_control, remotes_array);
    }
}