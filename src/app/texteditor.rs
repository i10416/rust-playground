// refs
// build own text editor: kilo.
// https://viewsourcecode.org/snaptoken/kilo/
/*
## Objective

antirez's kilo を少し改変したテキストエディタをRustで書く.

## Supported Features

 */

// user input -> stdin variable -> `program world`
//
use std::io::{self, Read};

#[link(name="enable_raw_mode.a")]
extern {
  pub fn enable_raw_mode()->u32;
}

fn main() -> Result<(), ()> {
    unsafe {
      enable_raw_mode();    
    }
    let buf = String::new();
    match read_rec(buf) {
        Ok(result) => {
            println!("result is {}", result);
            Ok(())
        }
        Err(_) => unimplemented!(),
    }
}
fn read_rec(mut buf: String) -> Result<String, String> {
    match io::stdin().read_to_string(&mut buf) {
        Ok(_) if buf.as_str().contains("q") => Ok(buf),
        Ok(_) => {
            read_rec(buf)
        }
        Err(_) => unimplemented!(),
    }
}



// todo: echo
// note: Rustのプログラムでは必ずCの標準ライブラリがリンクされている
// https://qiita.com/deta-mamoru/items/045c5569ebf2cf39c29e
/* 
use std::os::raw::c_int;
extern {
    fn abs(n: c_int) -> c_int;
}
fn main() {
    unsafe {
        println!("{}", abs(-1)); // 1
    }
}
*/
// externブロックの上に#[link(...)]属性を付けることで、特定のライブラリの関数を宣言することができる
/* 
#[link(name="libname")]
extern {
    fn fn_name();
}
*/
// user input -> stdin -> do something -> stdout
