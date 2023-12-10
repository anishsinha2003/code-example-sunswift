//! # A library which takes care of multiple handling cases
//!
//! ## Features
//!
//! - **handle:** responsible for handling the base case  (no remote connections i.e., -J tag)
//! - **run_server:** runs the server if there are remote conections
//! - **connect_to_remote:** responsible for connecting to the remote and running threads which take the input and prints output
//! - **connect_to_multiple_remotes:** same thing as connect_to_remote but if multiple remotes are given


use pars_libs::{parse_line, Remote, RemoteCommand};
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::process::ChildStdin;
use std::process::ChildStdout;
use std::sync::atomic::AtomicBool;
use std::sync::Condvar;
use std::sync::Mutex;
use std::thread::spawn;
use std::{
    io, process::Command, sync::atomic::Ordering, sync::Arc, sync::mpsc::channel
};

use helpers::{
    run_commands_line, my_notify, my_wait
};

#[derive(Debug)]
pub struct RemoteDetails {
    pub remote: Remote,
    pub concurrent_jobs: usize,
    pub child_stdin: Option<Arc<Mutex<ChildStdin>>>,
    pub child_stdout: Option<Arc<Mutex<ChildStdout>>>,
}

/// Given the arguments discussed below, handles user input, command-run, and output. Utilises threads.
///
/// # Arguments
///
/// * `concurrent_jobs` - a usize which represents the number concurrent jobs that can be used
/// * `termination_control` - the termination control represented by a string
/// * `client` - a flag which represents if its a client that's running this program
///
pub fn handle(concurrent_jobs: usize, termination_control: String, client: bool) {

    let (sender, receiver) = channel();
    let (sender_output, receiver_output) = channel();
    let command_fail: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let thread_count = Arc::new((Mutex::new(0), Condvar::new()));

    let input_thread = spawn(move || {
        let lines = io::stdin().lines();
        let thread_sender = sender.clone();
        for line in lines {
            match parse_line(line.unwrap().trim()) {
                Some(list_of_commands) => if let Ok(()) = thread_sender.send(list_of_commands) {},
                None => {
                    println!("Could not parse line!");
                    break;
                }
            }
        }
    });

    let worker_threads = spawn(move || {

        loop {
            let command_invalid_clone: Arc<AtomicBool> = Arc::clone(&command_fail);

            let list_of_commands = receiver.recv();
            let output_sender = sender_output.clone();

            // a wrapper for condvar-wait which behaves like a condavar-wait but also adds a new thread,
            // and waits until a thread is available for use. Once it is available, it will continue to
            // run the program from here onwards and create a new thread. After a thread is complete,
            // thread count is updated and other threads are notified (using my_notify)
            let thread_count_clone = Arc::clone(&thread_count);
            my_wait(thread_count_clone, concurrent_jobs);

            match list_of_commands.clone() {
                Err(_) => {
                    break;
                },
                _ => {
                }
            }

            if command_invalid_clone.load(Ordering::Relaxed) && termination_control == "lazy" {
                break;
            }
            else {
                let termination_control = termination_control.clone();
                let thread_count_clone = Arc::clone(&thread_count);
                spawn(move || {
                    let (output, command_fail) = run_commands_line(list_of_commands.clone().unwrap(),
                        command_invalid_clone.clone(), termination_control.clone());
                    let _ = output_sender.send(output);
                    if command_fail {
                        command_invalid_clone.store(true, Ordering::Relaxed);
                    }

                    // thread is finished, hence subtract thread count by 1 and
                    // notify that a thread is available
                    my_notify(thread_count_clone);
                });
            }
        }
    });

    // Receive and print results from the spawned threads
    let output_thread = spawn(move || {
        loop {
            let result = receiver_output.recv();
            if client {
                let mut stdout = io::stdout();
                if let Err(_) = result {
                    break;
                } else {
                    if result.clone().unwrap() != "" {
                        // change all newlines into ";", so when the new output is passed in the local,
                        // all we have to do is replace the ";", to \n and then print it as a whole
                        // this has to be done beause othewise the buffer only
                        // reads until new line (in server) which caused problems (bad design)
                        let modified = result.clone().unwrap().trim().replace("\n", ";");
                        writeln!(stdout, "{}", modified).expect("Failed to write to stdout");
                        stdout.flush().expect("Failed to flush stdout");
                    }
                }

            } else {
                if let Err(_) = result {
                    break;
                } else {
                    if result.clone().unwrap().trim() != "" {
                        println!("{}", result.clone().unwrap().trim());
                    }
                }

            }
        }
    });

    input_thread.join().unwrap();
    worker_threads.join().unwrap();
    output_thread.join().unwrap();
}

/// Given the arguments discussed below, handles the server side calls
/// different functions depending on number of remotes supplied
///
/// # Arguments
///
/// * `termination_control` - the termination control represented by a string
/// * `remotes` - list of RemotesDetails (checkout struct) to connect to
pub fn run_server(termination_control: String, remotes: Vec<RemoteDetails>) {
    // connect to single remote
    if remotes.len() == 1 {
        let remote_details = remotes.get(0).unwrap();
        let remote = &remote_details.remote;
        let concurrent_jobs = &remote_details.concurrent_jobs;
        connect_to_remote(
            remote.clone(),
            *concurrent_jobs,
            termination_control.clone(),
        );
    }

    // connect to multiple remotes
    if remotes.len() != 1 {
        connect_to_multiple_remotes(remotes, termination_control);
    }
}
/// Given the arguments discussed below, connects to a single remote passed
///
/// # Arguments
///
/// * `remote` - the remote to connect to
/// * `termination_control` - the termination control represented by a string
/// * `concurrent_jobs` - a usize which represents the number concurrent jobs that can be used
pub fn connect_to_remote(remote: Remote, concurrent_jobs: usize, termination_control: String) {
    let command_string = format!(
        "pars -J {} -e {} --client",
        concurrent_jobs, termination_control
    );
    let mut first_elem_command = Command::new(command_string);
    let mut cmd = first_elem_command
        .remote_spawn(&remote)
        .expect("Failed to start command");

    let mut child_stdin = cmd.stdin.take().unwrap();
    let mut child_stdout = cmd.stdout.take().unwrap();

    // channel to communicate between input and output
    // (used for tell each other when to break and continue)
    let (sender, receiver) = channel();

    let input_thread = spawn(move || loop {
        let mut input_data = String::new();
        match io::stdin().read_line(&mut input_data) {
            Ok(0) | Err(_) => {
                let sender = sender.clone();
                let _ = sender.send("break");
                break;
            }
            Ok(_) => {
                let sender = sender.clone();
                let _ = sender.send("continue");
                let _ = child_stdin.write_all(input_data.as_bytes());
                child_stdin.flush().unwrap();
            }
        }
    });

    let output_thread = spawn(move || loop {
        let continue_loop = receiver.recv();
        match continue_loop {
            Ok("continue") => {
                let mut client_output: Vec<u8> = Vec::new();
                let mut reader = BufReader::new(&mut child_stdout);
                let _ = reader.read_until(b'\n', &mut client_output);
                let modified = String::from_utf8(client_output)
                    .unwrap()
                    .trim()
                    .replace(';', "\n");
                if !modified.is_empty() {
                    println!("{}", modified);
                }
            }
            Ok("break") | Err(_) => break,
            _ => (),
        }
    });
    input_thread.join().unwrap();
    output_thread.join().unwrap();
}

/// Given the arguments discussed below, connects to multiple remote passed
///
/// # Arguments
///
/// * `remotes` - the remotes to connect to
/// * `termination_control` - the termination control represented by a string
pub fn connect_to_multiple_remotes(remotes: Vec<RemoteDetails>, termination_control: String) {

    let mut new_remotes: Vec<RemoteDetails> = Vec::new();
    let mut total_concurrent_jobs: usize = 0;

    for remote_details in remotes {
        let remote_detail = &remote_details.remote;
        let remote_concurrent_jobs = &remote_details.concurrent_jobs;
        let command_string = format!(
            "pars -J {} -e {} --client",
            remote_concurrent_jobs, termination_control
        );
        let mut first_elem_command = Command::new(command_string);
        let mut cmd = first_elem_command
            .remote_spawn(remote_detail)
            .expect("Failed to start command");

        let child_stdin = Arc::new(Mutex::new(cmd.stdin.take().unwrap()));
        let child_stdout = Arc::new(Mutex::new(cmd.stdout.take().unwrap()));

        let new_remote = RemoteDetails {
            remote: remote_detail.clone(),
            concurrent_jobs: *remote_concurrent_jobs,
            child_stdin: Some(child_stdin.clone()),
            child_stdout: Some(child_stdout.clone()),
        };
        // this will be used later on
        new_remotes.push(new_remote);

        total_concurrent_jobs = total_concurrent_jobs + remote_concurrent_jobs;
    }

    // channel to communicate between input and output
    // (used for tell each other when to break and continue)
    let (sender, receiver) = channel();
    let (sender_remote_stdout, receiver_remote_stdout) = channel();
    let (sender_remote_idx, receiver_remote_idx) = channel();

    let new_remotes_arc_mutex = Arc::new(Mutex::new(new_remotes));

    let arc_remotes_for_input = Arc::clone(&new_remotes_arc_mutex);
    let arc_remotes_for_output = Arc::clone(&new_remotes_arc_mutex);

    let thread_count = Arc::new((Mutex::new(0), Condvar::new()));

    let thread_count_input_clone = Arc::clone(&thread_count);
    let thread_count_output_clone = Arc::clone(&thread_count);

    let input_thread = spawn(move || {
        // first check which remote to use to run the command for stdin and stdout
        loop {
            let mut input_data = String::new();
            match io::stdin().read_line(&mut input_data) {
                Ok(0) | Err(_) => {
                    // tell the output thread to break
                    let sender = sender.clone();
                    let _ = sender.send("break");
                    break;
                }
                Ok(_) => {
                    // println!("{}", input_data);
                    let mut childstdin_to_use: Option<Arc<Mutex<ChildStdin>>> = None;
                    let mut childstdout_to_use: Option<Arc<Mutex<ChildStdout>>> = None;
                    let mut remote_idx = 0;

                    // loop through all the remotes and get which remote is available
                    my_wait(thread_count_input_clone.clone(), total_concurrent_jobs);

                    let mut remotes = arc_remotes_for_input.lock().unwrap();

                    for remote_check in &*remotes {
                        let threads_available_for_remote = remote_check.concurrent_jobs;
                        if threads_available_for_remote != 0 {
                            childstdin_to_use = Some(remote_check.child_stdin.clone().unwrap());
                            childstdout_to_use = Some(remote_check.child_stdout.clone().unwrap());
                            // a thread is being now used
                            remotes[remote_idx].concurrent_jobs -= 1;
                            drop(remotes);
                            break;
                        }
                        remote_idx += 1;
                    }
                    match childstdin_to_use {
                        Some(child_stdin) => {
                            // get all the sender clones first
                            let sender = sender.clone();
                            let sender_remote_stdout = sender_remote_stdout.clone();

                            // tell the output thread to break
                            let _ = sender.send("continue");

                            // tell the output whcih child tdout to use
                            let _ = sender_remote_stdout.send(childstdout_to_use);

                            // tell which remote to change (by giving the remote idx)
                            let _ = sender_remote_idx.send(remote_idx);

                            // flush the input into child stdout
                            let _ = child_stdin.lock().unwrap().write_all(input_data.as_bytes());
                            child_stdin.lock().unwrap().flush().unwrap();
                        },
                        None => {
                            // go back to the loop until a new thread is availble
                        }
                    }

                }
            }
        }
    });

    let output_thread = spawn(move || {
        let mut handlers = vec![];
        loop {
            // receive if we must continue to loop
            let continue_loop = receiver.recv();
            match continue_loop {
                Ok("continue") => {
                    // receive which child_stdout to use
                    let received_childstdout: Arc<Mutex<ChildStdout>> =
                        receiver_remote_stdout.recv().unwrap().unwrap();

                    // receieve which concurrent jobs of remote
                    // we have to change once the output has been printed
                    let remote_idx: usize = receiver_remote_idx.recv().unwrap();

                    let arc_remotes_for_output_clone = Arc::clone(&arc_remotes_for_output);
                    let thread_count_output_clone = Arc::clone(&thread_count_output_clone);
                    let handler = spawn(move || {
                        let mut child_stdout = received_childstdout.lock().unwrap();
                        let mut client_output: Vec<u8> = Vec::new();
                        let mut reader = BufReader::new(&mut *child_stdout);
                        let _ = reader.read_until(b'\n', &mut client_output);
                        let modified = String::from_utf8(client_output)
                            .unwrap()
                            .trim()
                            .replace(';', "\n");

                        // print the output
                        if !modified.is_empty() {
                            // once done, a thread output has been outputedd
                            // and we can notify that a thread is aviable and change remotes accordingly
                            println!("{}", modified);

                            let mut remotes = arc_remotes_for_output_clone.lock().unwrap();
                            remotes[remote_idx].concurrent_jobs += 1;
                            drop(remotes);
                            my_notify(thread_count_output_clone.clone());
                        }
                    });
                    handlers.push(handler);
                }
                Ok("break") | Err(_) => {
                    for handler in handlers {
                        handler.join().unwrap();
                    }
                    break;
                },
                _ => (),
            }
        }
    });

    input_thread.join().unwrap();
    output_thread.join().unwrap();
}