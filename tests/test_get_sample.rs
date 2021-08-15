use std::fs::File;

#[test]
fn test_get_sample_small() {
   let mut err = bootstrap::BS_Error::None;
   let mut test = File::open("C:/Users/caleb/Documents/Projects/bootstrap/tests/test_sample1.txt").unwrap();
   let sample = bootstrap::get_sample(&mut test, &mut err);
   assert_eq!(sample.unwrap(), vec!(1.0,2.0,3.0,4.0,5.0,6.0,7.0,8.0,9.0,10.0,11.0));
   let sample = bootstrap::get_sample(&mut "1,2,3,4, \r\n5,  \n \r ,,".as_bytes(), &mut err);
   assert_eq!(sample.unwrap(), vec!(1.0,2.0,3.0,4.0,5.0));
}

#[test]
fn test_get_sample_large() {
   let mut err = bootstrap::BS_Error::None;
   let mut test = File::open("C:/Users/caleb/Documents/Projects/bootstrap/tests/test_sample2.txt").unwrap();
   let sample = bootstrap::get_sample(&mut test, &mut err).unwrap();
   assert_eq!(sample[0], 33.0);
   assert_eq!(sample[sample.len() - 1], 961.0);
}

#[test]
fn test_bootstrap_mean() {
   let mut err = bootstrap::BS_Error::None;
   let mut test = File::open("C:/Users/caleb/Documents/Projects/bootstrap/tests/test_sample3.txt").unwrap();
   let bs_dist = bootstrap::bootstrap_mean(bootstrap::get_sample(&mut test, &mut err).unwrap());
   println!("size: {}, first: {}, last: {}", bs_dist.len(), bs_dist[0], bs_dist[bs_dist.len() - 1]);
}