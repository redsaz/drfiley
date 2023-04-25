pub mod walker;

pub fn public_function() {
    println!("called lib's `public_function()`");
    walker::public_function2();
}

