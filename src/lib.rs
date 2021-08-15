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

/// scales number of threads with amount of data to process, does not dictate
/// amount of data each thread processes
const BS_DATA_PER_THREAD: usize = 10000;

/// defined as resample_size / sample_size
const BS_RESAMPLE_RATIO: f32 = 0.5;

/// the minimum number of resamples to do
const BS_RESAMPLE_AMT: i32 = 100000;

// evolves an intermediate step with a new value in iterative statistic calculations
type bs_iter_evolve = fn(f64, f64) -> f64;
// finalizes an intermediate value for iterative statistic calculations
type bs_iter_finalize = fn(f64, f64) -> f64;
// computes a statistic from sample
type bs_compute = fn(& Vec<f64>) -> f64;

/// use str::from_utf8 and str::parse to attempt conversion
/// returns Result::Err on failure
#[macro_export]
macro_rules! utf8_to_f64 {
    ($utf8_slice:expr) => {
        match str::from_utf8($utf8_slice){
            Ok(data_str) => match data_str.parse::<f64>(){
                Ok(data) => Ok(data),
                Err(_) => Err("data not parsable as f64")
            },
            Err(_) => Err("expected utf-8")
        }
    };
}

pub enum BS_Error{
    None,
    DatumDropped
}

/// expects data interpretable as a floating point number and seperated
/// by any number of spaces, commas, newlines, or carriage returns.
/// According to the rust docs every call to Read::read can result in a system
/// call. Two buffers are used to avoid this.
/// When an invalid datum is encountered, it is dropped, parsing continues, err
/// is set
pub fn get_sample<R: Read>(source:&mut R, err:&mut BS_Error) -> Result<Vec<f64>, &'static str>
{
    let mut sample: Vec<f64> = Vec::new();
    let mut buf: [u8; BS_BUFSIZE] = [0 as u8; BS_BUFSIZE];
    let mut data: [u8; BS_DATASIZE] = [0 as u8; BS_DATASIZE];
    let mut didx: usize = 0;
    *err = BS_Error::None;
    loop{ match source.read(&mut buf){
        Ok(n) => {
            if n == 0 { break; } // EOF
            let mut m: usize = 0;
            while m < n { match buf[m]{
                b' ' | b',' | b'\n' | b'\r' => {
                    if didx > 0 {
                        match utf8_to_f64!(& data[0 .. didx]){
                            Ok(datum) => sample.push(datum),
                            Err(_) => *err = BS_Error::DatumDropped, // drop data point
                        }
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
        match utf8_to_f64!(& data[0 .. didx]){
            Ok(datum) => sample.push(datum),
            Err(_) => *err = BS_Error::DatumDropped, // drop data point
        }
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


/// sample being moved, and consumed, isn't desirable behavior because bootstrapping
/// the same sample multiple times for different statistics is a common use pattern.
/// it could be prevented by copying or by using atomic pointers.
///
/// mean, max, and min could be calculated with a function like the one below
pub fn bootstrap_iterative(sample: Vec<f64>, evolve: bs_iter_evolve, finalize: bs_iter_finalize) -> Result<Vec<f64>, String>
{
    // wrap sample in Arc so it can be distributed among threads
    let arc_sample = Arc::<Vec<f64>>::from(sample);
    // create thread safe data storage
    let mut bs_dist: Arc<Mutex<Vec<f64>>> = Arc::from(Mutex::from(Vec::new()));
    // keep track of threads
    let mut threads = Vec::<JoinHandle<()>>::new();

    // create threads
    for _ in 0 .. max(arc_sample.len() / BS_DATA_PER_THREAD, 15){
        let thread_arc_sample = arc_sample.clone();
        let thread_bs_dist = bs_dist.clone();
        threads.push( spawn(move || -> () {

            let mut rnd_gen = rand::thread_rng();
            for _ in 0 .. BS_RESAMPLE_AMT{

                let mut intm: f64 = 0.0;
                let resample_size: usize = (BS_RESAMPLE_RATIO * (thread_arc_sample.len() as f32)).floor() as usize;
                for _ in 0 .. resample_size{
                    intm += evolve(intm, thread_arc_sample[rnd_gen.gen::<usize>()])
                }
                // thread panics if mutex is poisoned
                thread_bs_dist.lock().unwrap().push(finalize(intm, resample_size as f64));
            }
        })
        )
    }

    for thread in threads{
        match thread.join(){
            Err(msg) => return Err(format!("child failed to join; {:?}", msg)),
            _ => continue
        }
    }

    return match Arc::try_unwrap(bs_dist){
        Ok(vec_in_mut) => Ok(vec_in_mut.into_inner().unwrap()),
        Err(msg) => Err(format!("undropped arc reference: {:?}", msg))
    }
}
