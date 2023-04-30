pub mod walker;

pub fn public_function() {
    eprintln!("called lib's `public_function()`");
    walker::public_function2();
}

