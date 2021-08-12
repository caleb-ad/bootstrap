use std::io::Read;
use std::vec::Vec;
use std::str;

const BS_BUFSIZE: usize = 1000;
const BS_DATASIZE: usize = 100;


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
pub fn get_sample<R>(source:&mut R) -> Result<Vec<f64>, &'static str>
where R: Read {
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
                    if didx >= BS_DATASIZE {return Err("each data point can be at most 100 bytes");}
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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
