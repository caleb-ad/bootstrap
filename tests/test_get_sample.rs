use std::fs::File;

#[test]
fn test_get_sample_small() {
   let mut test = File::open("C:/Users/caleb/Documents/Projects/bootstrap/tests/test_sample1.txt").unwrap();
   let sample = bootstrap::get_sample(&mut test);
   assert_eq!(sample.unwrap(), vec!(1.0,2.0,3.0,4.0,5.0,6.0,7.0,8.0,9.0,10.0,11.0));
   let sample = bootstrap::get_sample(&mut "1,2,3,4, \r\n5,  \n \r ,,".as_bytes());
   assert_eq!(sample.unwrap(), vec!(1.0,2.0,3.0,4.0,5.0));
}

#[test]
fn test_get_sample_large() {
   let mut test = File::open("C:/Users/caleb/Documents/Projects/bootstrap/tests/test_sample2.txt").unwrap();
   let sample = bootstrap::get_sample(&mut test).unwrap();
   assert_eq!(sample[0], 33.0);
   assert_eq!(sample[sample.len() - 1], 961.0);
}
