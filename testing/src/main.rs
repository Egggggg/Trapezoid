use glob::Pattern;
use walkdir::WalkDir;

fn main() {
    let search = Pattern::new("**").unwrap();

    let ignore_string = "
design
target
";
    let mut ignore: Vec<Pattern> = Vec::new();

    for line in ignore_string.split("\n") {
        ignore.push(Pattern::new(line).unwrap());
    }

    let entries = WalkDir::new("C://Users/bee/Projects/Trapezoid")
        .into_iter()
        .filter_entry(|e| {
            for pattern in &ignore {
                if pattern.matches(e.file_name().to_str().unwrap()) {
                    return false;
                }
            }

            return true;
        })
        .filter_map(|e| {
            if match e {
                Ok(_) => true,
                Err(_) => false,
            } && search.matches(e.as_ref().unwrap().file_name().to_str().unwrap())
            {
                return Some(e.unwrap());
            }

            return None;
        });

    for e in entries {
        println!("{}", e.path().display())
    }
}
