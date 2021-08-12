use std::io::Read;
use std::vec::Vec;
use std::str;

const bs_bufsize: usize = 1000;
const bs_datasize: usize = 100;

fn get_sample<R>(source:&mut R) -> Result<Vec<f64>, String>
where R: Read {
    let mut sample: Vec<f64> = Vec::new();
    let mut buf: [u8; bs_bufsize] = [0 as u8; bs_bufsize];
    let mut data: [u8; bs_datasize] = [0 as u8; bs_datasize];
    let mut didx: usize = 0;
    loop{ match source.read(&mut buf){
        Ok(n) => {
            if n == 0 { break; }
            let m: usize = 0;
            while m < n { match buf[m]{
                b' ' | b',' | b'\n' => {
                    if didx > 0{
                        sample.push(match str::from_utf8(& data[0..didx]){
                            Ok(data_str) => match data_str.parse::<f64>(){
                                Ok(data) => data,
                                Err(_) => return Err(String::from("data not parsable as f64"))
                            },
                            Err(_) => return Err(String::from("expected utf-8"))
                        })
                    }
                },
                _ => {
                    data[didx] = buf[m];
                    didx += 1;
                    if didx >= bs_datasize {return Err(String::from("each data point can be at most 100 characters"));}
                }
            } }
        },
        Err(err) => {
            return Err(err.to_string());
        }
    } }
    return Ok(sample);
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
