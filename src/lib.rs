use std::{
        io::Read,
        vec::Vec,
        str,
        sync::{
            Mutex,
            Arc},
        thread::{
            spawn,
            JoinHandle},
        cmp::max,
        panic
};
use rand::Rng;

const BS_BUFSIZE: usize = 1000;
const BS_DATASIZE: usize = 100;
const BS_DATA_PER_THREAD: usize = 10000;
const BS_RESAMPLE_RATIO: f32 = 0.5;
const BS_RESAMPLE_AMT: i32 = 100;

/// use str::from_utf8 and str::parse to attempt conversion
/// returns Result::Err on failure
#[macro_export]
macro_rules! utf8_to_f64 {
    ($utf8_slice:expr) => {
        match str::from_utf8($utf8_slice){
            Ok(data_str) => match data_str.parse::<f64>(){
                Ok(data) => data,
                Err(_) => return Err("data not parsable as f64")
            },
            Err(_) => return Err("expected utf-8")
        }
    };
}

/// expects data interpretable as a floating point number and seperated
/// by any number of spaces, commas, newlines, or carriage returns
pub fn get_sample<R: Read>(source:&mut R) -> Result<Vec<f64>, &'static str>
{
    let mut sample: Vec<f64> = Vec::new();
    let mut buf: [u8; BS_BUFSIZE] = [0 as u8; BS_BUFSIZE];
    let mut data: [u8; BS_DATASIZE] = [0 as u8; BS_DATASIZE];
    let mut didx: usize = 0;
    loop{ match source.read(&mut buf){
        Ok(n) => {
            if n == 0 { break; } // EOF
            let mut m: usize = 0;
            while m < n { match buf[m]{
                b' ' | b',' | b'\n' | b'\r' => {
                    if didx > 0{
                        sample.push(utf8_to_f64!(& data[0 .. didx]));
                        didx = 0;
                    }
                },
                _ => {
                    data[didx] = buf[m];
                    didx += 1;
                    if didx >= BS_DATASIZE {return Err("each data point can be at most BS_DATASIZE bytes");}
                }
            }; m += 1; }
        },
        Err(_) => {
            return Err("read failed");
        }
    } }
    if didx > 0 {
        sample.push(utf8_to_f64!(& data[0 .. didx]));
    }
    return Ok(sample);
}

pub fn bootstrap_mean(sample: Vec<f64>) -> Vec<f64>{
    let arc_sample: Arc<Vec<f64>> = Arc::from(sample);
    let bs_distribution = Arc::from(Mutex::from(Vec::<f64>::new()));
    let mut threads: Vec<JoinHandle<()>> = Vec::new();
    for _ in 0 .. max(arc_sample.len() % BS_DATA_PER_THREAD, 10){
        let temp_sample: Arc<Vec<f64>> = arc_sample.clone();
        let temp_bs_distribution = bs_distribution.clone();
        threads.push(spawn(move || {
            let mut rnd_gen = rand::thread_rng();
            for _ in 0 .. BS_RESAMPLE_AMT{
                let mut sum: f64 = 0.0;
                // below line is full of questionable behavior
                let resample_size: usize = (BS_RESAMPLE_RATIO * (temp_sample.len() as f32)).floor() as usize;
                for _ in 0 .. resample_size{
                    sum += temp_sample[(rnd_gen.gen::<usize>() as usize) % temp_sample.len()]
                }
                // no thread should panic and poison the mutex
                temp_bs_distribution.lock().unwrap().push(sum / (resample_size as f64))
            }
        }));
    }

    for thread in threads{
        match thread.join(){
            Err(msg) => panic!("child failed to join; {:?}", msg),
            _ => continue
        }
    }

    return match Arc::try_unwrap(bs_distribution){
        Ok(vec_in_mut) => vec_in_mut.into_inner().unwrap(),
        Err(_) => panic!("un-dropped arc reference")
    }
}

pub fn test() {
    let items = Arc::from(Mutex::new(vec!(1,2,3,4)));
    let mut items1 = items.clone();
    let child = spawn(move || {
        match items.lock(){
            Ok(mut item_vec) => {
                item_vec.push(5);
            },
            Err(_) => println!("poisoned child")
        }
    });

    match items1.lock(){
        Ok(mut item_vec) => {
            item_vec.push(6);
            println!("main: {:?}", item_vec);
        },
        Err(_) => println!("poisoned main")
    };

    match child.join(){
        Ok(_) => {
            println!("threads joined: {:?}", Arc::get_mut(&mut items1).unwrap().get_mut().unwrap());
        },
        Err(_) => {
            println!("child panicked");
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_test() {
        crate::test();
    }
}
