#[macro_export]
macro_rules! exit {
  ($($arg:tt)*) => {
    {
      eprintln!($($arg)*);
      exit(1);
    }
  };
}
