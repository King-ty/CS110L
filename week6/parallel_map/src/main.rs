use crossbeam_channel;
use std::{thread, time};

fn parallel_map<T, U, F>(input_vec: Vec<T>, num_threads: usize, f: F) -> Vec<U>
where
    F: FnOnce(T) -> U + Send + Copy + 'static,
    T: Send + 'static,
    U: Send + 'static + Default,
{
    let mut output_vec: Vec<U> = Vec::with_capacity(input_vec.len());
    // DONE: implement parallel map!
    output_vec.resize_with(input_vec.len(), Default::default);

    let (input_sender, input_receiver) = crossbeam_channel::unbounded();
    let (output_sender, output_receiver) = crossbeam_channel::unbounded();
    let mut threads = Vec::with_capacity(num_threads);
    // let index = 0;
    // let iter = input_vec.iter();
    for _ in 0..num_threads {
        let receiver = input_receiver.clone();
        let sender = output_sender.clone();
        threads.push(thread::spawn(move || {
            while let Ok((id, input)) = receiver.recv() {
                sender.send((id, f(input))).expect("No ouput receivers!");
            }
            // drop(sender); // 这里不需要手动drop
        }))
    }
    drop(output_sender);

    for (id, input) in input_vec.into_iter().enumerate() {
        input_sender.send((id, input)).expect("No input receivers!");
    }
    drop(input_sender);

    while let Ok((id, output)) = output_receiver.recv() {
        output_vec[id] = output;
    }

    for thread in threads {
        thread.join().expect("Panic occured in thread");
    }

    output_vec
}

fn main() {
    let v = vec![6, 7, 8, 9, 10, 1, 2, 3, 4, 5, 12, 18, 11, 5, 20];
    let squares = parallel_map(v, 10, |num| {
        println!("{} squared is {}", num, num * num);
        thread::sleep(time::Duration::from_millis(500));
        num * num
    });
    println!("squares: {:?}", squares);
}
