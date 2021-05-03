use crossbeam_channel;
use std::{thread, time};

fn parallel_map<T, U, F>(input_vec: Vec<T>, num_threads: usize, f: F) -> Vec<U>
where
    F: FnOnce(T) -> U + Send + Copy + 'static,
    T: Send + 'static,
    U: Send + 'static + Default,
{
    let mut output_vec: Vec<U> = Vec::with_capacity(input_vec.len());
    // TODO: in parallel, run f on each input element and collect the outputs,
    // in order, in output_vec
    
    let len = input_vec.len();
    let (sender1, receiver1) = crossbeam_channel::bounded( len );
    let (sender2, receiver2) = crossbeam_channel::bounded( len );
    for _ in 0..len {   // Initialize output_vec
        output_vec.push( Default::default() );
    }

    let mut threads_vec = Vec::new();
    for _ in 0..num_threads {
        let receiver1 = receiver1.clone();
        let sender2 = sender2.clone();
        threads_vec.push( thread::spawn( move || {
            while let Ok(input_pair) = receiver1.recv() {
                let (index, input) = input_pair;
                let output = f(input);
                let output_pair = (index, output);
                sender2.send(output_pair).expect("Bucket2 receiver dead?");
            }
            drop(sender2);
        } ) );
    }
    drop(sender2);

    // Send input numbers to buckets, then drop the sender
    for (index, item) in input_vec.into_iter().enumerate() {
        let input_pair = (index, item);
        sender1.send(input_pair).expect("Bucket1 receiver dead?");
    }
    drop(sender1);

    // receiver2 wrapped the results into ouput_vec
    while let Ok(output_pair) = receiver2.recv() {
        let (index, output) = output_pair;
        output_vec[index] = output;
    }

    for handle in threads_vec {
        handle.join().expect("Panic in joining child thread");
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
